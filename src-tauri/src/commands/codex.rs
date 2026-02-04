//! Codex 계정 관리 Tauri 커맨드

use crate::modules::codex::{
    add_codex_account as add_codex_account_internal, get_codex_account_usage, get_codex_active_account,
    import_from_codex_auth_json, load_codex_accounts, refresh_all_codex_usage,
    remove_codex_account, rename_codex_account, set_codex_active_account, start_codex_oauth_login,
    switch_to_codex_account, touch_codex_account, wait_for_codex_oauth_login, CodexAccount,
    CodexAccountInfo, CodexUsageInfo, OAuthLoginInfo,
};
use chrono::Utc;

/// Codex 계정 목록 조회
#[tauri::command]
pub async fn list_codex_accounts() -> Result<Vec<CodexAccountInfo>, String> {
    let store = load_codex_accounts()?;
    let active_id = store.active_account_id.as_deref();

    let accounts: Vec<CodexAccountInfo> = store
        .accounts
        .iter()
        .map(|a| CodexAccountInfo::from_account(a, active_id))
        .collect();

    Ok(accounts)
}

/// 현재 활성 Codex 계정 조회
#[tauri::command]
pub async fn get_active_codex_account() -> Result<Option<CodexAccountInfo>, String> {
    let store = load_codex_accounts()?;
    let active_id = store.active_account_id.as_deref();

    if let Some(active) = get_codex_active_account()? {
        Ok(Some(CodexAccountInfo::from_account(&active, active_id)))
    } else {
        Ok(None)
    }
}

/// 파일에서 Codex 계정 추가 (auth.json import)
#[tauri::command]
pub async fn add_codex_account_from_file(
    path: String,
    name: String,
) -> Result<CodexAccountInfo, String> {
    // auth.json에서 import
    let account = import_from_codex_auth_json(&path, name)?;

    // 저장소에 추가
    let stored = add_codex_account_internal(account)?;

    let store = load_codex_accounts()?;
    let active_id = store.active_account_id.as_deref();

    Ok(CodexAccountInfo::from_account(&stored, active_id))
}

/// Codex API 키 계정 추가
#[tauri::command]
pub async fn add_codex_account(label: Option<String>, api_key: String) -> Result<CodexAccountInfo, String> {
    let name = label
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("Codex-{}", Utc::now().timestamp()));

    let account = CodexAccount::new_api_key(name, api_key);
    let stored = add_codex_account_internal(account)?;

    let store = load_codex_accounts()?;
    let active_id = store.active_account_id.as_deref();

    Ok(CodexAccountInfo::from_account(&stored, active_id))
}

/// Codex 계정 전환
#[tauri::command]
pub async fn switch_codex_account(account_id: String) -> Result<(), String> {
    let store = load_codex_accounts()?;

    // 계정 찾기
    let account = store
        .accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| format!("계정을 찾을 수 없습니다: {}", account_id))?;

    // ~/.codex/auth.json에 작성
    switch_to_codex_account(account)?;

    // 활성 계정 설정
    set_codex_active_account(&account_id)?;

    // 마지막 사용 시간 업데이트
    touch_codex_account(&account_id)?;

    Ok(())
}

/// Codex 계정 삭제
#[tauri::command]
pub async fn delete_codex_account(account_id: String) -> Result<(), String> {
    remove_codex_account(&account_id)?;
    Ok(())
}

/// Codex 계정 이름 변경
#[tauri::command]
pub async fn rename_codex_account_cmd(account_id: String, new_name: String) -> Result<(), String> {
    rename_codex_account(&account_id, new_name)?;
    Ok(())
}

/// 단일 Codex 계정 사용량 조회
#[tauri::command]
pub async fn get_codex_usage(account_id: String) -> Result<CodexUsageInfo, String> {
    let store = load_codex_accounts()?;

    let account = store
        .accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| format!("계정을 찾을 수 없습니다: {}", account_id))?;

    get_codex_account_usage(account).await
}

/// (Legacy Alias) 단일 Codex 계정 사용량 새로고침 (get_codex_usage와 동일)
#[tauri::command]
pub async fn refresh_codex_account(account_id: String) -> Result<CodexUsageInfo, String> {
    get_codex_usage(account_id).await
}

/// 모든 Codex 계정 사용량 새로고침
#[tauri::command]
pub async fn refresh_all_codex_accounts_usage() -> Result<Vec<CodexUsageInfo>, String> {
    let store = load_codex_accounts()?;
    Ok(refresh_all_codex_usage(&store.accounts).await)
}

/// Codex OAuth 로그인 시작 (브라우저 열고 콜백 대기)
#[tauri::command]
pub async fn start_codex_oauth(account_name: String) -> Result<OAuthLoginInfo, String> {
    let (login_info, rx) = start_codex_oauth_login(account_name).await?;

    // 백그라운드에서 결과 대기
    tokio::spawn(async move {
        match wait_for_codex_oauth_login(rx).await {
            Ok(account) => {
                tracing::info!("[Codex OAuth] 계정 등록 완료: {}", account.name);
            }
            Err(e) => {
                tracing::error!("[Codex OAuth] 로그인 실패: {}", e);
            }
        }
    });

    Ok(login_info)
}

/// Codex OAuth 로그인 (동기식 - 완료까지 대기)
#[tauri::command]
pub async fn start_codex_oauth_and_wait(account_name: String) -> Result<CodexAccountInfo, String> {
    let (_, rx) = start_codex_oauth_login(account_name).await?;

    // 로그인 완료 대기
    let account = wait_for_codex_oauth_login(rx).await?;

    let store = load_codex_accounts()?;
    let active_id = store.active_account_id.as_deref();

    Ok(CodexAccountInfo::from_account(&account, active_id))
}
