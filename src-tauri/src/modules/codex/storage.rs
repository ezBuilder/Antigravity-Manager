//! Codex 계정 저장소 관리

use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use super::types::{CodexAccount, CodexAccountsStore};

const CODEX_DIR: &str = "codex";
const ACCOUNTS_FILE: &str = "accounts.json";

/// Codex 데이터 디렉토리 경로 반환
pub fn get_codex_data_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "홈 디렉토리를 찾을 수 없습니다".to_string())?;

    let data_dir = home.join(".antigravity_tools").join(CODEX_DIR);

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Codex 데이터 디렉토리 생성 실패: {}", e))?;
    }

    Ok(data_dir)
}

/// 계정 저장소 파일 경로 반환
fn get_accounts_file_path() -> Result<PathBuf, String> {
    Ok(get_codex_data_dir()?.join(ACCOUNTS_FILE))
}

/// 계정 목록 로드
pub fn load_codex_accounts() -> Result<CodexAccountsStore, String> {
    let path = get_accounts_file_path()?;

    if !path.exists() {
        return Ok(CodexAccountsStore::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("계정 파일 읽기 실패: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("계정 파일 파싱 실패: {}", e))
}

/// 계정 목록 저장
pub fn save_codex_accounts(store: &CodexAccountsStore) -> Result<(), String> {
    let path = get_accounts_file_path()?;

    let content =
        serde_json::to_string_pretty(store).map_err(|e| format!("계정 직렬화 실패: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("계정 파일 저장 실패: {}", e))?;

    Ok(())
}

/// 계정 추가
pub fn add_codex_account(account: CodexAccount) -> Result<CodexAccount, String> {
    let mut store = load_codex_accounts()?;

    // 중복 이메일 체크
    if let Some(email) = &account.email {
        if store
            .accounts
            .iter()
            .any(|a| a.email.as_ref() == Some(email))
        {
            return Err(format!("이미 등록된 이메일입니다: {}", email));
        }
    }

    store.accounts.push(account.clone());
    save_codex_accounts(&store)?;

    Ok(account)
}

/// 계정 삭제
pub fn remove_codex_account(account_id: &str) -> Result<(), String> {
    let mut store = load_codex_accounts()?;

    let initial_len = store.accounts.len();
    store.accounts.retain(|a| a.id != account_id);

    if store.accounts.len() == initial_len {
        return Err(format!("계정을 찾을 수 없습니다: {}", account_id));
    }

    // 활성 계정이 삭제되면 초기화
    if store.active_account_id.as_deref() == Some(account_id) {
        store.active_account_id = None;
    }

    save_codex_accounts(&store)?;

    Ok(())
}

/// 활성 계정 설정
pub fn set_codex_active_account(account_id: &str) -> Result<(), String> {
    let mut store = load_codex_accounts()?;

    // 계정 존재 확인
    if !store.accounts.iter().any(|a| a.id == account_id) {
        return Err(format!("계정을 찾을 수 없습니다: {}", account_id));
    }

    store.active_account_id = Some(account_id.to_string());
    save_codex_accounts(&store)?;

    Ok(())
}

/// 활성 계정 가져오기
pub fn get_codex_active_account() -> Result<Option<CodexAccount>, String> {
    let store = load_codex_accounts()?;

    if let Some(active_id) = &store.active_account_id {
        Ok(store.accounts.iter().find(|a| &a.id == active_id).cloned())
    } else {
        Ok(None)
    }
}

/// 계정 마지막 사용 시간 업데이트
pub fn touch_codex_account(account_id: &str) -> Result<(), String> {
    let mut store = load_codex_accounts()?;

    if let Some(account) = store.accounts.iter_mut().find(|a| a.id == account_id) {
        account.last_used_at = Some(Utc::now());
        save_codex_accounts(&store)?;
    }

    Ok(())
}

/// 계정 이름 변경
pub fn rename_codex_account(account_id: &str, new_name: String) -> Result<(), String> {
    let mut store = load_codex_accounts()?;

    if let Some(account) = store.accounts.iter_mut().find(|a| a.id == account_id) {
        account.name = new_name;
        save_codex_accounts(&store)?;
        Ok(())
    } else {
        Err(format!("계정을 찾을 수 없습니다: {}", account_id))
    }
}
