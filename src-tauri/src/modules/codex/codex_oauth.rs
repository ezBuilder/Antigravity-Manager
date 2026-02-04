//! Codex OAuth 로그인 - ChatGPT OAuth 플로우
//! 참고: https://github.com/Lampese/codex-switcher

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;

use super::storage;
use super::types::{CodexAccount, CodexAuthData};

/// OpenAI Auth0 설정 (Codex CLI와 동일)
const DEFAULT_ISSUER: &str = "https://auth.openai.com";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann"; // Codex CLI client_id
const DEFAULT_PORT: u16 = 1455; // Codex CLI와 동일 포트

/// PKCE 코드
#[derive(Debug, Clone)]
pub struct PkceCodes {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// PKCE 코드 생성
pub fn generate_pkce() -> PkceCodes {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 64];
    rng.fill_bytes(&mut bytes);

    let code_verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);

    PkceCodes {
        code_verifier,
        code_challenge,
    }
}

/// 랜덤 state 파라미터 생성
fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// OAuth 인증 URL 생성
pub fn build_authorize_url(
    issuer: &str,
    client_id: &str,
    redirect_uri: &str,
    pkce: &PkceCodes,
    state: &str,
) -> String {
    let params = [
        ("response_type", "code"),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("scope", "openid profile email offline_access"),
        ("code_challenge", &pkce.code_challenge),
        ("code_challenge_method", "S256"),
        ("id_token_add_organizations", "true"),
        ("codex_cli_simplified_flow", "true"),
        ("state", state),
        ("originator", "codex_cli_rs"), // OpenAI OAuth 필수
    ];

    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{issuer}/oauth/authorize?{query_string}")
}

/// 토큰 응답
#[derive(Debug, Clone, serde::Deserialize)]
struct TokenResponse {
    id_token: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct CodexRefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub expires_in: Option<i64>,
}

/// authorization code로 토큰 교환
async fn exchange_code_for_tokens(
    issuer: &str,
    client_id: &str,
    redirect_uri: &str,
    pkce: &PkceCodes,
    code: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();

    let body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        urlencoding::encode(code),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(client_id),
        urlencoding::encode(&pkce.code_verifier)
    );

    let resp = client
        .post(format!("{issuer}/oauth/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("토큰 교환 요청 실패: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("토큰 교환 실패: {} - {}", status, body));
    }

    let tokens: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("토큰 파싱 실패: {}", e))?;

    Ok(tokens)
}

/// refresh_token으로 access_token 갱신
async fn refresh_codex_access_token(refresh_token: &str) -> Result<CodexRefreshResult, String> {
    let client = reqwest::Client::new();

    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}&scope={}",
        urlencoding::encode(refresh_token),
        urlencoding::encode(CLIENT_ID),
        urlencoding::encode("openid profile email offline_access")
    );

    let resp = client
        .post(format!("{DEFAULT_ISSUER}/oauth/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("토큰 갱신 요청 실패: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("토큰 갱신 실패: {} - {}", status, body));
    }

    let payload: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("토큰 갱신 응답 파싱 실패: {}", e))?;

    let access_token = payload
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("refresh 응답에 access_token이 없습니다")?
        .to_string();
    let id_token = payload.get("id_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let refresh_token = payload
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let expires_in = payload.get("expires_in").and_then(|v| v.as_i64());

    let (email, plan_type, chatgpt_account_id) = id_token
        .as_deref()
        .map(parse_id_token_claims)
        .unwrap_or((None, None, None));

    Ok(CodexRefreshResult {
        access_token,
        refresh_token,
        id_token,
        email,
        plan_type,
        chatgpt_account_id,
        expires_in,
    })
}

/// Codex 계정 토큰 갱신 후 저장
pub async fn refresh_codex_account_tokens(
    account_id: &str,
) -> Result<(CodexAccount, CodexRefreshResult), String> {
    let mut store = storage::load_codex_accounts()?;
    let account = store
        .accounts
        .iter_mut()
        .find(|a| a.id == account_id)
        .ok_or_else(|| format!("계정을 찾을 수 없습니다: {}", account_id))?;

    let refresh_token = match &account.auth_data {
        CodexAuthData::ChatGPT { refresh_token, .. } => refresh_token.clone(),
        CodexAuthData::ApiKey { .. } => {
            return Err("API 키 계정은 토큰 갱신을 지원하지 않습니다".to_string());
        }
    };

    let refresh_result = refresh_codex_access_token(&refresh_token).await?;

    if let CodexAuthData::ChatGPT {
        access_token,
        refresh_token,
        id_token,
        account_id: chatgpt_account_id,
    } = &mut account.auth_data
    {
        *access_token = refresh_result.access_token.clone();
        if let Some(new_refresh) = &refresh_result.refresh_token {
            *refresh_token = new_refresh.clone();
        }
        if let Some(new_id) = &refresh_result.id_token {
            *id_token = new_id.clone();
        }
        if let Some(new_chatgpt_id) = &refresh_result.chatgpt_account_id {
            *chatgpt_account_id = Some(new_chatgpt_id.clone());
        }
    }

    if let Some(email) = &refresh_result.email {
        account.email = Some(email.clone());
    }
    if let Some(plan_type) = &refresh_result.plan_type {
        account.plan_type = Some(plan_type.clone());
    }

    storage::save_codex_accounts(&store)?;

    Ok((account.clone(), refresh_result))
}

/// JWT ID 토큰에서 클레임 추출
fn parse_id_token_claims(id_token: &str) -> (Option<String>, Option<String>, Option<String>) {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return (None, None, None);
    }

    let payload = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(bytes) => bytes,
        Err(_) => return (None, None, None),
    };

    let json: serde_json::Value = match serde_json::from_slice(&payload) {
        Ok(v) => v,
        Err(_) => return (None, None, None),
    };

    let email = json.get("email").and_then(|v| v.as_str()).map(String::from);

    let auth_claims = json.get("https://api.openai.com/auth");

    let plan_type = auth_claims
        .and_then(|auth| auth.get("chatgpt_plan_type"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let account_id = auth_claims
        .and_then(|auth| auth.get("chatgpt_account_id"))
        .and_then(|v| v.as_str())
        .map(String::from);

    (email, plan_type, account_id)
}

/// OAuth 로그인 정보
#[derive(Debug, Clone, serde::Serialize)]
pub struct OAuthLoginInfo {
    pub auth_url: String,
    pub callback_port: u16,
}

/// OAuth 플로우 상태
#[allow(dead_code)]
struct OAuthFlowState {
    pkce: PkceCodes,
    state: String,
    redirect_uri: String,
    account_name: String,
    tx: Option<oneshot::Sender<Result<CodexAccount, String>>>,
}

static OAUTH_FLOW_STATE: OnceLock<Mutex<Option<OAuthFlowState>>> = OnceLock::new();

fn get_oauth_flow_state() -> &'static Mutex<Option<OAuthFlowState>> {
    OAUTH_FLOW_STATE.get_or_init(|| Mutex::new(None))
}

/// OAuth 로그인 시작
pub async fn start_codex_oauth_login(
    account_name: String,
) -> Result<
    (
        OAuthLoginInfo,
        oneshot::Receiver<Result<CodexAccount, String>>,
    ),
    String,
> {
    let pkce = generate_pkce();
    let state = generate_state();

    tracing::info!("[Codex OAuth] 로그인 시작: {}", account_name);

    // HTTP 서버 시작
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT))
        .map_err(|e| format!("OAuth 서버 시작 실패: {}", e))?;

    let actual_port = listener
        .local_addr()
        .map(|a| a.port())
        .map_err(|e| format!("포트 확인 실패: {}", e))?;

    let redirect_uri = format!("http://localhost:{}/auth/callback", actual_port);
    let auth_url = build_authorize_url(DEFAULT_ISSUER, CLIENT_ID, &redirect_uri, &pkce, &state);

    tracing::info!("[Codex OAuth] 서버 포트: {}", actual_port);
    tracing::info!("[Codex OAuth] 인증 URL: {}", auth_url);

    let login_info = OAuthLoginInfo {
        auth_url: auth_url.clone(),
        callback_port: actual_port,
    };

    let (tx, rx) = oneshot::channel();

    // 상태 저장
    {
        let mut guard = get_oauth_flow_state().lock().unwrap();
        *guard = Some(OAuthFlowState {
            pkce: pkce.clone(),
            state: state.clone(),
            redirect_uri: redirect_uri.clone(),
            account_name: account_name.clone(),
            tx: Some(tx),
        });
    }

    // 백그라운드 스레드에서 HTTP 서버 실행
    let pkce_clone = pkce.clone();
    let state_clone = state.clone();
    let redirect_uri_clone = redirect_uri.clone();
    let account_name_clone = account_name.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(run_oauth_server(
            listener,
            pkce_clone,
            state_clone,
            redirect_uri_clone,
            account_name_clone,
        ));
    });

    // 브라우저 열기
    let _ = webbrowser::open(&auth_url);

    Ok((login_info, rx))
}

/// OAuth 콜백 서버 실행
async fn run_oauth_server(
    listener: std::net::TcpListener,
    pkce: PkceCodes,
    expected_state: String,
    redirect_uri: String,
    account_name: String,
) {
    use std::io::{Read, Write};

    listener.set_nonblocking(true).ok();
    let timeout = Duration::from_secs(300); // 5분 타임아웃
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            tracing::warn!("[Codex OAuth] 로그인 타임아웃");
            send_oauth_result(Err("로그인 타임아웃".to_string()));
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buffer = [0u8; 4096];
                if stream.read(&mut buffer).is_err() {
                    continue;
                }

                let request = String::from_utf8_lossy(&buffer);

                // GET /auth/callback?code=...&state=... 파싱
                if let Some(path_line) = request.lines().next() {
                    if path_line.contains("/auth/callback") {
                        let result = handle_callback(
                            path_line,
                            &pkce,
                            &expected_state,
                            &redirect_uri,
                            &account_name,
                        )
                        .await;

                        let html = match &result {
                            Ok(_) => success_html(),
                            Err(e) => error_html(e),
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();

                        send_oauth_result(result);
                        break;
                    }
                }

                // 404 응답
                let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
                let _ = stream.write_all(response.as_bytes());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// OAuth 결과 전송
fn send_oauth_result(result: Result<CodexAccount, String>) {
    let mut guard = get_oauth_flow_state().lock().unwrap();
    if let Some(state) = guard.take() {
        if let Some(tx) = state.tx {
            let _ = tx.send(result);
        }
    }
}

/// 콜백 처리
async fn handle_callback(
    path_line: &str,
    pkce: &PkceCodes,
    expected_state: &str,
    redirect_uri: &str,
    account_name: &str,
) -> Result<CodexAccount, String> {
    // URL 파싱
    let url_str = if let Some(start) = path_line.find('/') {
        let end = path_line.rfind(' ').unwrap_or(path_line.len());
        format!("http://localhost{}", &path_line[start..end])
    } else {
        return Err("잘못된 요청".to_string());
    };

    let parsed = url::Url::parse(&url_str).map_err(|e| format!("URL 파싱 실패: {}", e))?;

    let params: HashMap<String, String> = parsed.query_pairs().into_owned().collect();

    tracing::info!(
        "[Codex OAuth] 콜백 수신: {:?}",
        params.keys().collect::<Vec<_>>()
    );

    // 에러 체크
    if let Some(error) = params.get("error") {
        let error_desc = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("알 수 없는 오류");
        return Err(format!("OAuth 에러: {} - {}", error, error_desc));
    }

    // state 검증
    if params.get("state").map(String::as_str) != Some(expected_state) {
        return Err("state 불일치".to_string());
    }

    // code 추출
    let code = params
        .get("code")
        .filter(|c| !c.is_empty())
        .ok_or("authorization code 없음")?;

    tracing::info!("[Codex OAuth] 토큰 교환 중...");

    // 토큰 교환
    let tokens =
        exchange_code_for_tokens(DEFAULT_ISSUER, CLIENT_ID, redirect_uri, pkce, code).await?;

    tracing::info!("[Codex OAuth] 토큰 교환 성공!");

    // ID 토큰에서 클레임 추출
    let (email, plan_type, chatgpt_account_id) = parse_id_token_claims(&tokens.id_token);

    // 계정 생성
    let account = CodexAccount::new_chatgpt(
        account_name.to_string(),
        email,
        plan_type,
        tokens.id_token,
        tokens.access_token,
        tokens.refresh_token,
        chatgpt_account_id,
    );

    // 저장소에 추가
    storage::add_codex_account(account.clone())?;

    tracing::info!("[Codex OAuth] 계정 등록 완료: {}", account_name);

    Ok(account)
}

/// OAuth 로그인 대기
pub async fn wait_for_codex_oauth_login(
    rx: oneshot::Receiver<Result<CodexAccount, String>>,
) -> Result<CodexAccount, String> {
    rx.await
        .map_err(|_| "OAuth 로그인이 취소되었습니다".to_string())?
}

/// 성공 HTML
fn success_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>로그인 성공</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }
        .container { text-align: center; background: white; padding: 40px 60px; border-radius: 16px; box-shadow: 0 20px 60px rgba(0,0,0,0.3); }
        h1 { color: #333; margin-bottom: 10px; }
        p { color: #666; }
        .checkmark { font-size: 48px; margin-bottom: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">✓</div>
        <h1>로그인 성공!</h1>
        <p>이 창을 닫고 Antigravity Manager로 돌아가세요.</p>
    </div>
</body>
</html>"#.to_string()
}

/// 에러 HTML
fn error_html(error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>로그인 실패</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: linear-gradient(135deg, #ff6b6b 0%, #c44569 100%); }}
        .container {{ text-align: center; background: white; padding: 40px 60px; border-radius: 16px; box-shadow: 0 20px 60px rgba(0,0,0,0.3); }}
        h1 {{ color: #c44569; margin-bottom: 10px; }}
        p {{ color: #666; }}
        .error {{ font-size: 48px; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error">✕</div>
        <h1>로그인 실패</h1>
        <p>{}</p>
    </div>
</body>
</html>"#,
        error
    )
}
