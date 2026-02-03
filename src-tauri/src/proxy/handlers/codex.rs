//! Codex Proxy Handler
//! ChatGPT 계정의 토큰으로 OpenAI API에 gpt-5.2-codex / gpt-5.1-codex-max / gpt-5.1-codex-mini 호출

use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use tracing::{debug, info};
use futures::StreamExt;

use crate::modules::codex::{storage, types::CodexAuthData};
use crate::proxy::server::AppState;

/// OpenAI API 베이스 URL
const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// Codex 공식 모델: 요청 모델이 이 중 하나면 그대로 전달, 아니면 기본값 사용
const CODEX_MODELS: &[&str] = &["gpt-5.2-codex", "gpt-5.1-codex-max", "gpt-5.1-codex-mini"];
const CODEX_DEFAULT_MODEL: &str = "gpt-5.2-codex";

/// 모델명으로 Codex 사용 여부 판단
pub fn should_use_codex(model: &str) -> bool {
    if CODEX_MODELS.contains(&model) {
        return true;
    }
    let model_lower = model.to_lowercase();
    if model_lower.starts_with("gpt-5") && model_lower.contains("codex") {
        return true;
    }
    if model_lower.starts_with("codex") {
        return true;
    }
    false
}

fn resolve_codex_model(request_model: &str) -> &'static str {
    for m in CODEX_MODELS {
        if *m == request_model {
            return m;
        }
    }
    CODEX_DEFAULT_MODEL
}

/// Codex API 호출 (비스트리밍). Anthropic 핸들러에서 재사용.
/// Returns (status, response_body_json, model_used).
pub async fn call_codex_chat_api(
    body: Value,
) -> Result<(StatusCode, Value, String), (StatusCode, String)> {
    let _trace_id = format!("codex_{}", chrono::Utc::now().timestamp_subsec_millis());
    let (access_token, _account_id, chatgpt_account_id) = get_active_codex_token().await?;

    let original_model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let model_to_send = resolve_codex_model(&original_model);
    let mut body = body;
    body["model"] = json!(model_to_send);
    body["stream"] = json!(false);

    const CODEX_USER_AGENT: &str = "codex-cli/1.0.0";
    let client = reqwest::Client::new();
    let mut req_builder = client
        .post(format!("{}/chat/completions", OPENAI_API_BASE))
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, CODEX_USER_AGENT)
        .json(&body);
    if let Some(cg_id) = &chatgpt_account_id {
        req_builder = req_builder.header("chatgpt-account-id", cg_id.as_str());
    }
    let response = req_builder
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("OpenAI API 요청 실패: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        let error_body: Value = if let Ok(parsed) = serde_json::from_str::<Value>(&error_text) {
            if parsed.get("error").is_some() {
                parsed
            } else {
                json!({ "error": { "message": error_text, "type": "api_error", "code": "internal_error" } })
            }
        } else {
            json!({ "error": { "message": error_text, "type": "api_error", "code": "internal_error" } })
        };
        return Ok((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            error_body,
            model_to_send.to_string(),
        ));
    }

    let response_body: Value = response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("응답 파싱 실패: {}", e)))?;
    Ok((StatusCode::OK, response_body, model_to_send.to_string()))
}

/// Codex 채팅 요청 처리 (OpenAI API 방식)
pub async fn handle_codex_chat(
    State(_state): State<AppState>,
    Json(mut body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let trace_id = format!("codex_{}", chrono::Utc::now().timestamp_subsec_millis());
    info!("[{}] Codex API Request", trace_id);

    let (access_token, account_id, chatgpt_account_id) = get_active_codex_token().await?;

    let original_model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let model_to_send = resolve_codex_model(&original_model);
    body["model"] = json!(model_to_send);

    debug!(
        "[{}] Model: {} → {}, account: {}",
        trace_id, original_model, model_to_send, account_id
    );

    let stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);

    if stream {
        const CODEX_USER_AGENT: &str = "codex-cli/1.0.0";
        let client = reqwest::Client::new();
        let mut req_builder = client
            .post(format!("{}/chat/completions", OPENAI_API_BASE))
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, CODEX_USER_AGENT)
            .json(&body);
        if let Some(cg_id) = &chatgpt_account_id {
            req_builder = req_builder.header("chatgpt-account-id", cg_id.as_str());
        }
        let response = req_builder
            .send()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("OpenAI API 요청 실패: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error_body: Value = if let Ok(parsed) = serde_json::from_str::<Value>(&error_text) {
                if parsed.get("error").is_some() {
                    parsed
                } else {
                    json!({ "error": { "message": error_text, "type": "api_error", "code": "internal_error" } })
                }
            } else {
                json!({ "error": { "message": error_text, "type": "api_error", "code": "internal_error" } })
            };
            return Ok((
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
                Json(error_body),
            )
            .into_response());
        }

        let body = Body::from_stream(response.bytes_stream());
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("X-Codex-Account", &account_id)
            .header("X-Model", model_to_send)
            .body(body)
            .unwrap();
        return Ok(response.into_response());
    }

    // 비스트리밍: 공통 API 호출 후 응답만 래핑
    match call_codex_chat_api(body).await {
        Ok((status, response_body, model_used)) => Ok((
            status,
            [
                ("X-Codex-Account", account_id.as_str()),
                ("X-Model", model_used.as_str()),
            ],
            Json(response_body),
        )
            .into_response()),
        Err(e) => Err(e),
    }
}

/// 활성 Codex 계정의 토큰 가져오기
/// Returns (access_token, antigravity_account_id, chatgpt_account_id_for_header)
async fn get_active_codex_token() -> Result<(String, String, Option<String>), (StatusCode, String)> {
    let active = storage::get_codex_active_account().map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("활성 계정 없음: {}", e),
        )
    })?;

    match active {
        Some(account) => match &account.auth_data {
            CodexAuthData::ChatGPT {
                access_token,
                account_id: cg_id,
                ..
            } => Ok((
                access_token.clone(),
                account.id.clone(),
                cg_id.clone(),
            )),
            CodexAuthData::ApiKey { key } => {
                Ok((key.clone(), account.id.clone(), None))
            }
        },
        None => {
            let store = storage::load_codex_accounts().map_err(|e| {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("계정 로드 실패: {}", e),
                )
            })?;

            if store.accounts.is_empty() {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "등록된 Codex 계정이 없습니다".to_string(),
                ));
            }

            let first = &store.accounts[0];
            match &first.auth_data {
                CodexAuthData::ChatGPT {
                    access_token,
                    account_id: cg_id,
                    ..
                } => Ok((
                    access_token.clone(),
                    first.id.clone(),
                    cg_id.clone(),
                )),
                CodexAuthData::ApiKey { key } => Ok((key.clone(), first.id.clone(), None)),
            }
        }
    }
}
