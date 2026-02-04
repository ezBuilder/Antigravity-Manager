use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

use crate::proxy::common::model_mapping::is_codex_model;
use crate::proxy::config::{PmRouterConfig, PmRouterScope};
use crate::proxy::mappers::claude::models::{ClaudeRequest, MessageContent, ContentBlock, SystemPrompt};
use crate::proxy::server::AppState;

const ROUTER_ALLOWED_MODELS: &[&str] = &[
    "gpt-5.2-codex",
    "gpt-5.1-codex-max",
    "gpt-5.1-codex-mini",
    "claude-sonnet-4-5",
    "claude-sonnet-4-5-thinking",
    "claude-opus-4-5-thinking",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-thinking",
    "gemini-2.5-flash-lite",
    "gemini-3-flash",
    "gemini-3-pro-high",
    "gemini-3-pro-low",
    "gemini-3-pro-image",
];

const ROUTER_PROMPT_TEMPLATE: &str = r#"You are the PM Router agent for Antigravity.
Your job is to choose the BEST model for the task and return strict JSON.

RULES (priority):
1) Code implementation quality/CLI workflows -> prefer gpt-5.2-codex, fallback claude-sonnet-4-5, then gemini-2.5-pro.
2) Deep debugging/root cause analysis -> prefer claude-sonnet-4-5-thinking, fallback gpt-5.1-codex-max, then gemini-2.5-pro.
3) Code review/security/testing -> prefer claude-sonnet-4-5, fallback gpt-5.2-codex, then gemini-2.5-pro.
4) Architecture/ADR/high-risk changes -> prefer claude-opus-4-5-thinking, fallback gpt-5.1-codex-max, then claude-sonnet-4-5-thinking.
5) Docs/PRD/summary -> prefer claude-sonnet-4-5, fallback gpt-5.1-codex-mini, then gemini-2.5-flash.
6) Research/comparison -> prefer gemini-2.5-pro, fallback claude-sonnet-4-5, then gpt-5.1-codex-mini.
7) Image/UI/diagram -> prefer gemini-3-pro-image, fallback gemini-2.5-pro, then gpt-5.2-codex.
8) High-volume low-risk -> prefer gemini-2.5-flash or gemini-3-flash.
9) Avoid thinking/max unless needed. If you choose a thinking/max model, set needs_pro=true.

Available model ids:
{{model_list}}

Task context:
- requested_model: {{requested_model}}
- client_user_agent: {{user_agent}}
- has_images: {{has_images}}
- has_tools: {{has_tools}}
- system_prompt: {{system_prompt}}
- recent_messages: {{recent_messages}}

Return ONLY valid JSON:
{
  "selected_model": "model-id",
  "task_type": "coding|debugging|review|architecture|docs|research|image|general",
  "needs_pro": true|false,
  "reason": "short reason"
}
"#;

#[derive(Debug, Deserialize)]
struct RouterResponse {
    selected_model: String,
    #[serde(default)]
    task_type: String,
    #[serde(default)]
    needs_pro: bool,
    #[serde(default)]
    reason: String,
}

#[derive(Debug, Clone)]
pub struct RouterDecision {
    pub selected_model: String,
    pub reason: String,
    pub task_type: String,
    pub used_router_model: String,
    pub used_pro: bool,
}

pub fn should_apply_router(config: &PmRouterConfig, headers: &HeaderMap) -> bool {
    if !config.enabled {
        return false;
    }

    match config.scope {
        PmRouterScope::AllRequests => true,
        PmRouterScope::CliOnly => {
            let user_agent = headers
                .get(axum::http::header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let ua_lower = user_agent.to_lowercase();
            config
                .cli_user_agents
                .iter()
                .any(|needle| ua_lower.contains(&needle.to_lowercase()))
        }
    }
}

pub fn should_escalate_to_pro(config: &PmRouterConfig, context: &str) -> bool {
    if config.pro_keywords.is_empty() {
        return false;
    }
    let lower = context.to_lowercase();
    config
        .pro_keywords
        .iter()
        .any(|kw| lower.contains(&kw.to_lowercase()))
}

/// Codex 토큰 부재 등으로 라우터 모델 호출이 실패했는지 판별
fn is_claude_model(model: &str) -> bool {
    model.to_lowercase().starts_with("claude-")
}

async fn call_router_with_fallback(
    state: &AppState,
    primary_model: &str,
    fallback_model: &str,
    prompt: &str,
    trace_id: &str,
    label: &str,
) -> Result<(String, String), String> {
    match call_router_model(state, primary_model, prompt).await {
        Ok(response) => Ok((response, primary_model.to_string())),
        Err(err) => {
            if primary_model == fallback_model {
                return Err(err);
            }
            warn!(
                "[{}][PM-Router] {} model {} failed: {}. Falling back to {}.",
                trace_id, label, primary_model, err, fallback_model
            );
            let response = call_router_model(state, fallback_model, prompt).await?;
            Ok((response, fallback_model.to_string()))
        }
    }
}

pub async fn select_model_for_claude_request(
    state: &AppState,
    config: &PmRouterConfig,
    request: &ClaudeRequest,
    headers: &HeaderMap,
    trace_id: &str,
) -> Result<RouterDecision, String> {
    let context = build_router_context(request, config.max_context_chars);
    let prompt = build_router_prompt(request, headers, &context);

    let (lite_response, used_lite_model) =
        call_router_with_fallback(
            state,
            &config.pm_lite_model,
            &config.fallback_model,
            &prompt,
            trace_id,
            "PM-lite",
        )
        .await?;

    let parsed_lite = parse_router_response(&lite_response)?;
    let mut selected = validate_router_model(&parsed_lite.selected_model, config);
    let mut used_router_model = used_lite_model;
    let mut used_pro = false;

    if parsed_lite.needs_pro || should_escalate_to_pro(config, &context) {
        let pro_prompt = build_router_prompt(request, headers, &context);
        let (pro_response, used_pro_model_opt) =
            match call_router_with_fallback(
                state,
                &config.pm_pro_model,
                &config.fallback_model,
                &pro_prompt,
                trace_id,
                "PM-pro",
            )
            .await
            {
                Ok((response, model_used)) => (response, Some(model_used)),
                Err(err) => {
                    warn!(
                        "[{}][PM-Router] PM-pro failed: {} (falling back to PM-lite)",
                        trace_id, err
                    );
                    (String::new(), None)
                }
            };
        if let Some(pro_model_used) = used_pro_model_opt {
            if let Ok(parsed_pro) = parse_router_response(&pro_response) {
                selected = validate_router_model(&parsed_pro.selected_model, config);
                used_router_model = pro_model_used;
                used_pro = true;
                info!(
                    "[{}][PM-Router] Escalated to PM-pro ({} -> {})",
                    trace_id, config.pm_lite_model, config.pm_pro_model
                );
            }
        }
    }

    if is_codex_model(&selected) {
        warn!(
            "[{}][PM-Router] Selected Codex model ({}) is not supported for Claude protocol. Falling back to {}.",
            trace_id,
            selected,
            config.fallback_model
        );
        selected = config.fallback_model.clone();
    }

    Ok(RouterDecision {
        selected_model: selected,
        reason: parsed_lite.reason,
        task_type: parsed_lite.task_type,
        used_router_model,
        used_pro,
    })
}

fn build_router_prompt(request: &ClaudeRequest, headers: &HeaderMap, context: &str) -> String {
    let model_list = ROUTER_ALLOWED_MODELS.join(", ");
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let has_images = request.messages.iter().any(|msg| message_has_image(&msg.content));
    let has_tools = request.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false);
    let system_prompt = render_system_prompt(&request.system);

    ROUTER_PROMPT_TEMPLATE
        .replace("{{model_list}}", &model_list)
        .replace("{{requested_model}}", &request.model)
        .replace("{{user_agent}}", user_agent)
        .replace("{{has_images}}", &has_images.to_string())
        .replace("{{has_tools}}", &has_tools.to_string())
        .replace("{{system_prompt}}", &system_prompt)
        .replace("{{recent_messages}}", context)
}

fn build_router_context(request: &ClaudeRequest, max_chars: usize) -> String {
    let mut chunks = Vec::new();

    for msg in request.messages.iter().rev().take(6).rev() {
        let content = extract_message_text(&msg.content);
        if !content.is_empty() {
            chunks.push(format!("{}: {}", msg.role, content));
        }
    }

    let mut context = chunks.join("\n");
    if context.chars().count() > max_chars {
        context = context.chars().take(max_chars).collect::<String>();
        context.push_str("…");
    }
    context
}

fn extract_message_text(content: &MessageContent) -> String {
    match content {
        MessageContent::String(s) => s.clone(),
        MessageContent::Array(arr) => {
            let mut out = Vec::new();
            for block in arr {
                match block {
                    ContentBlock::Text { text } => out.push(text.clone()),
                    ContentBlock::ToolUse { name, .. } => out.push(format!("[tool_use:{}]", name)),
                    ContentBlock::ToolResult { .. } => out.push("[tool_result]".to_string()),
                    ContentBlock::Image { .. } => out.push("[image]".to_string()),
                    ContentBlock::Document { .. } => out.push("[document]".to_string()),
                    ContentBlock::Thinking { .. } => {}
                    ContentBlock::RedactedThinking { .. } => {}
                    ContentBlock::ServerToolUse { name, .. } => out.push(format!("[server_tool:{}]", name)),
                    ContentBlock::WebSearchToolResult { .. } => out.push("[web_search_result]".to_string()),
                }
            }
            out.join(" ")
        }
    }
}

fn message_has_image(content: &MessageContent) -> bool {
    match content {
        MessageContent::String(_) => false,
        MessageContent::Array(arr) => arr.iter().any(|block| matches!(block, ContentBlock::Image { .. })),
    }
}

fn render_system_prompt(system: &Option<SystemPrompt>) -> String {
    match system {
        Some(SystemPrompt::String(s)) => s.clone(),
        Some(SystemPrompt::Array(arr)) => arr.iter().map(|b| b.text.clone()).collect::<Vec<_>>().join("\n"),
        None => "-".to_string(),
    }
}

fn validate_router_model(selected: &str, config: &PmRouterConfig) -> String {
    let trimmed = selected.trim();
    if ROUTER_ALLOWED_MODELS.contains(&trimmed) {
        trimmed.to_string()
    } else {
        config.fallback_model.clone()
    }
}

fn parse_router_response(response: &str) -> Result<RouterResponse, String> {
    let cleaned = response.trim();
    let json_str = if let Some(start) = cleaned.find('{') {
        if let Some(end) = cleaned.rfind('}') {
            &cleaned[start..=end]
        } else {
            cleaned
        }
    } else {
        cleaned
    };
    serde_json::from_str::<RouterResponse>(json_str).map_err(|e| format!("Router JSON parse error: {}", e))
}

async fn call_router_model(
    state: &AppState,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    if is_codex_model(model) {
        call_openai_router_model(state, model, prompt).await
    } else if is_claude_model(model) {
        call_anthropic_router_model(state, model, prompt).await
    } else {
        call_gemini_router_model(state, model, prompt).await
    }
}

async fn call_openai_router_model(
    state: &AppState,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let token_manager = state.token_manager.clone();
    let (mut api_key, _project_id, _email, _wait_ms) = token_manager
        .get_token("codex", false, None, model)
        .await?;

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": "Return ONLY JSON." },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.2,
        "max_tokens": 256
    });

    let client = crate::utils::http::get_long_client();
    let mut chatgpt_account_id = token_manager.find_codex_chatgpt_account_id(&api_key);

    let mut req = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(axum::http::header::USER_AGENT, "codex-cli/1.0.0")
        .bearer_auth(&api_key)
        .json(&body);
    if let Some(cg_id) = &chatgpt_account_id {
        req = req.header("chatgpt-account-id", cg_id);
    }

    let mut resp = req
        .send()
        .await
        .map_err(|e| format!("OpenAI router request failed: {}", e))?;

    if matches!(resp.status(), axum::http::StatusCode::UNAUTHORIZED | axum::http::StatusCode::FORBIDDEN) {
        if let Ok(Some(refreshed)) = token_manager.refresh_codex_token_by_access_token(&api_key).await {
            api_key = refreshed.access_token;
            chatgpt_account_id = refreshed.chatgpt_account_id;
            let mut retry_req = client
                .post("https://api.openai.com/v1/chat/completions")
                .header(axum::http::header::USER_AGENT, "codex-cli/1.0.0")
                .bearer_auth(&api_key)
                .json(&body);
            if let Some(cg_id) = &chatgpt_account_id {
                retry_req = retry_req.header("chatgpt-account-id", cg_id);
            }
            resp = retry_req
                .send()
                .await
                .map_err(|e| format!("OpenAI router request failed after refresh: {}", e))?;
        }
    }

    let status = resp.status();
    let payload: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("OpenAI router invalid response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "OpenAI router error {}: {}",
            status,
            payload
        ));
    }

    payload["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "OpenAI router missing content".to_string())
}

async fn call_gemini_router_model(
    state: &AppState,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let token_manager = state.token_manager.clone();
    let (access_token, project_id, _email, _wait_ms) = token_manager
        .get_token("agent", false, None, model)
        .await?;

    let body = json!({
        "project": project_id,
        "requestId": format!("pm-router-{}", uuid::Uuid::new_v4()),
        "request": {
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": prompt }]
                }
            ],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 256
            }
        },
        "model": model,
        "userAgent": "antigravity",
        "requestType": "agent"
    });

    let response = state
        .upstream
        .call_v1_internal("generateContent", &access_token, body, None)
        .await
        .map_err(|e| format!("Gemini router request failed: {}", e))?;

    let status = response.status();
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Gemini router invalid response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Gemini router error {}: {}", status, payload));
    }

    payload["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Gemini router missing content".to_string())
}

async fn call_anthropic_router_model(
    state: &AppState,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let zai = state.zai.read().await.clone();
    if !zai.enabled || zai.api_key.trim().is_empty() {
        return Err("Anthropic router unavailable: z.ai is disabled or missing api_key".to_string());
    }

    let body = json!({
        "model": model,
        "system": "Return ONLY JSON.",
        "messages": [
            {
                "role": "user",
                "content": [{ "type": "text", "text": prompt }]
            }
        ],
        "temperature": 0.2,
        "max_tokens": 256
    });

    let timeout_secs = state.request_timeout.max(5);
    let upstream_proxy = state.upstream_proxy.read().await.clone();
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs));
    if let Some(config) = upstream_proxy {
        if config.enabled && !config.url.is_empty() {
            let proxy = reqwest::Proxy::all(&config.url)
                .map_err(|e| format!("Invalid upstream proxy url: {}", e))?;
            builder = builder.proxy(proxy);
        }
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to build Anthropic router client: {}", e))?;

    let url = format!("{}/v1/messages", zai.base_url.trim_end_matches('/'));
    let resp = client
        .post(url)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header("x-api-key", zai.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic router request failed: {}", e))?;

    let status = resp.status();
    let payload: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Anthropic router invalid response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Anthropic router error {}: {}", status, payload));
    }

    payload["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Anthropic router missing content".to_string())
}
