//! Codex 계정 관리용 타입 정의

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Codex 계정 저장소
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountsStore {
    /// 스키마 버전
    pub version: u32,
    /// 계정 목록
    pub accounts: Vec<CodexAccount>,
    /// 현재 활성 계정 ID
    pub active_account_id: Option<String>,
}

impl Default for CodexAccountsStore {
    fn default() -> Self {
        Self {
            version: 1,
            accounts: Vec::new(),
            active_account_id: None,
        }
    }
}

/// Codex 계정 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccount {
    /// 고유 ID (UUID)
    pub id: String,
    /// 사용자 정의 이름
    pub name: String,
    /// 이메일 (ChatGPT 인증 시)
    pub email: Option<String>,
    /// 플랜 타입: free, plus, pro, team 등
    pub plan_type: Option<String>,
    /// 인증 모드
    pub auth_mode: CodexAuthMode,
    /// 인증 데이터
    pub auth_data: CodexAuthData,
    /// 생성 시간
    pub created_at: DateTime<Utc>,
    /// 마지막 사용 시간
    pub last_used_at: Option<DateTime<Utc>>,
}

impl CodexAccount {
    /// API 키로 새 계정 생성
    pub fn new_api_key(name: String, api_key: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email: None,
            plan_type: None,
            auth_mode: CodexAuthMode::ApiKey,
            auth_data: CodexAuthData::ApiKey { key: api_key },
            created_at: Utc::now(),
            last_used_at: None,
        }
    }

    /// ChatGPT OAuth로 새 계정 생성
    pub fn new_chatgpt(
        name: String,
        email: Option<String>,
        plan_type: Option<String>,
        id_token: String,
        access_token: String,
        refresh_token: String,
        account_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            plan_type,
            auth_mode: CodexAuthMode::ChatGPT,
            auth_data: CodexAuthData::ChatGPT {
                id_token,
                access_token,
                refresh_token,
                account_id,
            },
            created_at: Utc::now(),
            last_used_at: None,
        }
    }
}

/// 인증 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexAuthMode {
    /// OpenAI API 키
    ApiKey,
    /// ChatGPT OAuth 토큰
    ChatGPT,
}

/// 인증 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexAuthData {
    /// API 키 인증
    ApiKey { key: String },
    /// ChatGPT OAuth 인증
    ChatGPT {
        id_token: String,
        access_token: String,
        refresh_token: String,
        account_id: Option<String>,
    },
}

/// Codex auth.json 파일 형식
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAuthJson {
    #[serde(rename = "OPENAI_API_KEY", skip_serializing_if = "Option::is_none")]
    pub openai_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<CodexTokenData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,
}

/// 토큰 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexTokenData {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

/// 프론트엔드용 계정 정보 (민감 정보 제외)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccountInfo {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub auth_mode: CodexAuthMode,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl CodexAccountInfo {
    pub fn from_account(account: &CodexAccount, active_id: Option<&str>) -> Self {
        Self {
            id: account.id.clone(),
            name: account.name.clone(),
            email: account.email.clone(),
            plan_type: account.plan_type.clone(),
            auth_mode: account.auth_mode,
            is_active: active_id == Some(&account.id),
            created_at: account.created_at,
            last_used_at: account.last_used_at,
        }
    }
}

/// 사용량 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexUsageInfo {
    pub account_id: String,
    pub plan_type: Option<String>,
    /// 1차 Rate Limit 사용률 (0-100%)
    pub primary_used_percent: Option<f64>,
    /// 1차 윈도우 시간 (분)
    pub primary_window_minutes: Option<i64>,
    /// 1차 리셋 시간 (unix timestamp)
    pub primary_resets_at: Option<i64>,
    /// 2차 Rate Limit 사용률 (0-100%)
    pub secondary_used_percent: Option<f64>,
    /// 2차 윈도우 시간 (분)
    pub secondary_window_minutes: Option<i64>,
    /// 2차 리셋 시간 (unix timestamp)
    pub secondary_resets_at: Option<i64>,
    /// 크레딧 보유 여부
    pub has_credits: Option<bool>,
    /// 무제한 크레딧 여부
    pub unlimited_credits: Option<bool>,
    /// 크레딧 잔액
    pub credits_balance: Option<String>,
    /// 에러 메시지
    pub error: Option<String>,
}

impl CodexUsageInfo {
    pub fn error(account_id: String, error: String) -> Self {
        Self {
            account_id,
            plan_type: None,
            primary_used_percent: None,
            primary_window_minutes: None,
            primary_resets_at: None,
            secondary_used_percent: None,
            secondary_window_minutes: None,
            secondary_resets_at: None,
            has_credits: None,
            unlimited_credits: None,
            credits_balance: None,
            error: Some(error),
        }
    }
}

/// API 응답 - Rate Limit 상태
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitStatusPayload {
    pub plan_type: String,
    #[serde(default)]
    pub rate_limit: Option<RateLimitDetails>,
    #[serde(default)]
    pub credits: Option<CreditStatusDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitDetails {
    pub primary_window: Option<RateLimitWindow>,
    pub secondary_window: Option<RateLimitWindow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitWindow {
    pub used_percent: f64,
    pub limit_window_seconds: Option<i32>,
    pub reset_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreditStatusDetails {
    pub has_credits: bool,
    pub unlimited: bool,
    #[serde(default)]
    pub balance: Option<String>,
}
