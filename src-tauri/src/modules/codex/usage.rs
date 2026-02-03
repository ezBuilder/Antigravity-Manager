//! Codex 사용량 조회 API

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};

use super::types::{
    CodexAccount, CodexAuthData, CodexUsageInfo, CreditStatusDetails, RateLimitDetails,
    RateLimitStatusPayload, RateLimitWindow,
};

const CHATGPT_BACKEND_API: &str = "https://chatgpt.com/backend-api";

/// 계정 사용량 조회
pub async fn get_codex_account_usage(account: &CodexAccount) -> Result<CodexUsageInfo, String> {
    tracing::info!("[Codex Usage] 계정 사용량 조회: {}", account.name);

    match &account.auth_data {
        CodexAuthData::ApiKey { .. } => {
            // API 키 계정은 사용량 조회 불가
            Ok(CodexUsageInfo {
                account_id: account.id.clone(),
                plan_type: Some("api_key".to_string()),
                primary_used_percent: None,
                primary_window_minutes: None,
                primary_resets_at: None,
                secondary_used_percent: None,
                secondary_window_minutes: None,
                secondary_resets_at: None,
                has_credits: None,
                unlimited_credits: None,
                credits_balance: None,
                error: Some("API 키 계정은 사용량 조회를 지원하지 않습니다".to_string()),
            })
        }
        CodexAuthData::ChatGPT {
            access_token,
            account_id,
            ..
        } => {
            get_usage_with_chatgpt_token(
                &account.id,
                &account.name,
                access_token,
                account_id.as_deref(),
            )
            .await
        }
    }
}

/// ChatGPT 토큰으로 사용량 조회
async fn get_usage_with_chatgpt_token(
    account_id: &str,
    account_name: &str,
    access_token: &str,
    chatgpt_account_id: Option<&str>,
) -> Result<CodexUsageInfo, String> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("codex-cli/1.0.0"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|e| format!("잘못된 access token: {}", e))?,
    );

    if let Some(acc_id) = chatgpt_account_id {
        if let Ok(header_name) = HeaderName::from_bytes(b"chatgpt-account-id") {
            if let Ok(header_value) = HeaderValue::from_str(acc_id) {
                headers.insert(header_name, header_value);
            }
        }
    }

    let url = format!("{CHATGPT_BACKEND_API}/wham/usage");

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("사용량 조회 요청 실패: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::warn!("[Codex Usage] API 에러: {} - {}", status, body);
        return Ok(CodexUsageInfo::error(
            account_id.to_string(),
            format!("API 에러: {}", status),
        ));
    }

    let body_text = response
        .text()
        .await
        .map_err(|e| format!("응답 읽기 실패: {}", e))?;

    let payload: RateLimitStatusPayload =
        serde_json::from_str(&body_text).map_err(|e| format!("응답 파싱 실패: {}", e))?;

    let usage = convert_payload_to_usage_info(account_id, payload);

    tracing::info!(
        "[Codex Usage] {} - primary: {:?}%, plan: {:?}",
        account_name,
        usage.primary_used_percent,
        usage.plan_type
    );

    Ok(usage)
}

/// API 응답을 UsageInfo로 변환
fn convert_payload_to_usage_info(
    account_id: &str,
    payload: RateLimitStatusPayload,
) -> CodexUsageInfo {
    let (primary, secondary) = extract_rate_limits(payload.rate_limit);
    let credits = extract_credits(payload.credits);

    CodexUsageInfo {
        account_id: account_id.to_string(),
        plan_type: Some(payload.plan_type),
        primary_used_percent: primary.as_ref().map(|w| w.used_percent),
        primary_window_minutes: primary
            .as_ref()
            .and_then(|w| w.limit_window_seconds)
            .map(|s| (i64::from(s) + 59) / 60),
        primary_resets_at: primary.as_ref().and_then(|w| w.reset_at),
        secondary_used_percent: secondary.as_ref().map(|w| w.used_percent),
        secondary_window_minutes: secondary
            .as_ref()
            .and_then(|w| w.limit_window_seconds)
            .map(|s| (i64::from(s) + 59) / 60),
        secondary_resets_at: secondary.as_ref().and_then(|w| w.reset_at),
        has_credits: credits.as_ref().map(|c| c.has_credits),
        unlimited_credits: credits.as_ref().map(|c| c.unlimited),
        credits_balance: credits.and_then(|c| c.balance),
        error: None,
    }
}

fn extract_rate_limits(
    rate_limit: Option<RateLimitDetails>,
) -> (Option<RateLimitWindow>, Option<RateLimitWindow>) {
    match rate_limit {
        Some(details) => (details.primary_window, details.secondary_window),
        None => (None, None),
    }
}

fn extract_credits(credits: Option<CreditStatusDetails>) -> Option<CreditStatusDetails> {
    credits
}

/// 모든 계정의 사용량을 병렬로 조회
pub async fn refresh_all_codex_usage(accounts: &[CodexAccount]) -> Vec<CodexUsageInfo> {
    tracing::info!("[Codex Usage] {} 계정의 사용량 조회 시작", accounts.len());

    let futures: Vec<_> = accounts
        .iter()
        .map(|account| async move {
            match get_codex_account_usage(account).await {
                Ok(info) => info,
                Err(e) => {
                    tracing::warn!("[Codex Usage] {} 에러: {}", account.name, e);
                    CodexUsageInfo::error(account.id.clone(), e)
                }
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    tracing::info!("[Codex Usage] 조회 완료");

    results
}
