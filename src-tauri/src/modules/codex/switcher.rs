//! Codex 계정 전환 로직 - ~/.codex/auth.json 파일 관리

use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use super::types::{CodexAccount, CodexAuthData, CodexAuthJson, CodexTokenData};

/// Codex 홈 디렉토리 경로 (~/.codex)
pub fn get_codex_home() -> Result<PathBuf, String> {
    // CODEX_HOME 환경변수 우선
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        return Ok(PathBuf::from(codex_home));
    }

    let home = dirs::home_dir().ok_or_else(|| "홈 디렉토리를 찾을 수 없습니다".to_string())?;

    Ok(home.join(".codex"))
}

/// Codex auth.json 파일 경로
#[allow(dead_code)]
pub fn get_codex_auth_file() -> Result<PathBuf, String> {
    Ok(get_codex_home()?.join("auth.json"))
}

/// 계정 전환 - ~/.codex/auth.json 파일 교체
pub fn switch_to_codex_account(account: &CodexAccount) -> Result<(), String> {
    let codex_home = get_codex_home()?;

    // .codex 디렉토리 생성
    if !codex_home.exists() {
        fs::create_dir_all(&codex_home).map_err(|e| format!(".codex 디렉토리 생성 실패: {}", e))?;
    }

    let auth_json = create_auth_json(account)?;
    let auth_path = codex_home.join("auth.json");

    let content = serde_json::to_string_pretty(&auth_json)
        .map_err(|e| format!("auth.json 직렬화 실패: {}", e))?;

    fs::write(&auth_path, content).map_err(|e| format!("auth.json 저장 실패: {}", e))?;

    // Unix에서 권한 설정 (600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&auth_path, perms).map_err(|e| format!("권한 설정 실패: {}", e))?;
    }

    tracing::info!("[Codex] 계정 전환 완료: {}", account.name);

    Ok(())
}

/// CodexAccount에서 auth.json 구조 생성
fn create_auth_json(account: &CodexAccount) -> Result<CodexAuthJson, String> {
    match &account.auth_data {
        CodexAuthData::ApiKey { key } => Ok(CodexAuthJson {
            openai_api_key: Some(key.clone()),
            tokens: None,
            last_refresh: None,
        }),
        CodexAuthData::ChatGPT {
            id_token,
            access_token,
            refresh_token,
            account_id,
        } => Ok(CodexAuthJson {
            openai_api_key: None,
            tokens: Some(CodexTokenData {
                id_token: id_token.clone(),
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                account_id: account_id.clone(),
            }),
            last_refresh: Some(Utc::now()),
        }),
    }
}

/// 기존 auth.json 파일에서 계정 import
pub fn import_from_codex_auth_json(
    path: &str,
    account_name: String,
) -> Result<CodexAccount, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("auth.json 읽기 실패: {}", e))?;

    let auth: CodexAuthJson =
        serde_json::from_str(&content).map_err(|e| format!("auth.json 파싱 실패: {}", e))?;

    // 인증 모드 결정
    if let Some(api_key) = auth.openai_api_key {
        Ok(CodexAccount::new_api_key(account_name, api_key))
    } else if let Some(tokens) = auth.tokens {
        // ID 토큰에서 이메일/플랜 추출
        let (email, plan_type) = parse_id_token_claims(&tokens.id_token);

        Ok(CodexAccount::new_chatgpt(
            account_name,
            email,
            plan_type,
            tokens.id_token,
            tokens.access_token,
            tokens.refresh_token,
            tokens.account_id,
        ))
    } else {
        Err("auth.json에 API 키 또는 토큰이 없습니다".to_string())
    }
}

/// JWT ID 토큰에서 클레임 추출 (검증 없이)
fn parse_id_token_claims(id_token: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return (None, None);
    }

    // 페이로드 디코딩 (두 번째 부분)
    use base64::Engine;
    let payload = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(bytes) => bytes,
        Err(_) => return (None, None),
    };

    let json: serde_json::Value = match serde_json::from_slice(&payload) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };

    let email = json.get("email").and_then(|v| v.as_str()).map(String::from);

    let plan_type = json
        .get("https://api.openai.com/auth")
        .and_then(|auth| auth.get("chatgpt_plan_type"))
        .and_then(|v| v.as_str())
        .map(String::from);

    (email, plan_type)
}

/// 현재 auth.json 읽기
#[allow(dead_code)]
pub fn read_current_codex_auth() -> Result<Option<CodexAuthJson>, String> {
    let path = get_codex_auth_file()?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("auth.json 읽기 실패: {}", e))?;

    let auth: CodexAuthJson =
        serde_json::from_str(&content).map_err(|e| format!("auth.json 파싱 실패: {}", e))?;

    Ok(Some(auth))
}

/// 활성 로그인 여부 확인
#[allow(dead_code)]
pub fn has_active_codex_login() -> Result<bool, String> {
    match read_current_codex_auth()? {
        Some(auth) => Ok(auth.openai_api_key.is_some() || auth.tokens.is_some()),
        None => Ok(false),
    }
}
