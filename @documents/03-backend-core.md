# 03. 백엔드(Core) 기능 상세

이 문서는 Tauri( Rust ) 측의 **핵심 모듈과 Command API**를 설명합니다. 프론트엔드 UI는 이 명령들을 호출해 모든 기능을 수행합니다.

## 1) 주요 진입점 및 초기화

- `src-tauri/src/main.rs`
  - OS별 렌더링 문제 회피 설정(Linux WebKit DMABUF).
  - `antigravity_tools_lib::run()`으로 앱 실행.

- `src-tauri/src/lib.rs`
  - **Headless 모드 처리**(환경변수로 API Key/관리자 비밀번호/인증 모드 주입).
  - **로거 초기화**, **토큰 통계 DB 초기화**, **보안 DB 초기화**.
  - Desktop 모드에서 Tauri 플러그인 초기화 및 Tray 생성.
  - 관리 서버/프록시 서버 자동 기동 및 스케줄러 시작.
  - 방대한 Tauri 명령(Command) 등록.

## 2) 계정 관리 (modules/account.rs, modules/account_service.rs)

### 계정 저장 구조

- `~/.antigravity_tools/accounts/*.json`에 계정 JSON 저장
- `accounts.json` 인덱스 파일에 요약 정보 및 현재 계정 저장
- 계정 파일 입출력 시 **락(ACCOUNT_INDEX_LOCK)**으로 동시 쓰기 충돌 방지

### 주요 기능

- **계정 추가/삭제/일괄 삭제/재정렬**
- **현재 계정 전환 및 마지막 사용 시간 업데이트**
- **토큰/쿼터 갱신 시 계정 JSON 갱신**
- **계정 내보내기(backup)**

## 3) OAuth 흐름 (modules/oauth, modules/oauth_server)

- UI에서 OAuth URL 생성 → 브라우저 인증 → 로컬 콜백 처리.
- Headless 환경에서도 `submit_oauth_code`로 수동 제출 가능.

## 4) 마이그레이션/동기화 (modules/migration)

- V1 DB에서 계정 가져오기
- IDE/기존 DB(state.vscdb)에서 계정 동기화
- 외부 경로 지정(import_custom_db) 지원

## 5) 장치 지문(Device Fingerprint) 관리 (modules/device)

- 기존 `storage.json`에서 machineId/macMachineId/devDeviceId/sqmId 추출
- 새 프로필 생성/저장/적용 및 히스토리 관리
- 계정별 프로필 바인딩 및 원복 기능 제공

## 6) 설정 관리 (modules/config, models/config)

- 언어/테마/자동 새로고침/자동 동기화
- 프록시 설정(포트/키/라우팅/보안/로그 등)
- 스케줄러(워밍업), 쿼터 보호, 핀 고정 모델, 회로 차단기 설정

## 7) 스케줄러 & 워밍업 (modules/scheduler, modules/quota)

- 일정 주기로 계정 쿼터를 스캔하여 **100% 모델에 워밍업 요청**
- 최근 워밍업 기록(warmup_history.json) 기반 중복 방지
- 계정별/전체 워밍업 명령 제공

## 8) 업데이트/자동 실행 (modules/update_checker, commands/autostart)

- GitHub 릴리즈 기반 업데이트 확인
- 자동 업데이트 체크 주기 저장
- OS별 자동 실행(enable/disable) 지원

## 9) HTTP API 서버 설정 (modules/http_api)

- 외부 프로그램 연동용 HTTP API 설정을 저장/로드
- `http_api_settings.json`에 활성화 여부/포트 저장 (기본 19527)

## 10) Codex 계정 관리 (modules/codex)

- Codex 계정은 별도 저장소(`~/.antigravity_tools/codex/accounts.json`)에 저장
- OAuth/토큰 갱신/사용량 조회/이름 변경 등 명령 제공
- 프록시 TokenManager가 Codex 계정도 풀에 포함해 라우팅 가능

## 11) Cloudflared 터널 관리 (modules/cloudflared, commands/cloudflared)

- Cloudflared 설치/상태 확인/시작/정지 명령 제공
- 프록시 설정 화면에서 터널 상태를 제어하고 외부 접속 경로를 확보

## 12) Tray/시스템 통합 (modules/tray, modules/integration)

- 트레이 메뉴에서 계정 전환/새로고침 이벤트를 UI와 동기화
- 데스크톱/헤드리스 환경을 구분해 시스템 연동 로직을 분기

## 13) 로그/캐시/프로세스 관리

- 로그 파일 관리 및 클리어
- 로그는 `~/.antigravity_tools/logs/` 하위에 날짜별로 저장
- Antigravity 캐시 정리 기능 제공
- 실행 중인 Antigravity 경로 및 실행 인자 탐지

## 14) 디버그 콘솔/로그 브리지 (modules/log_bridge)

- Rust 로그를 프론트 DebugConsole로 전달
- UI에서 디버그 콘솔 활성화/비활성화 및 로그 조회

## 15) 토큰 통계 DB (modules/token_stats)

- 요청 단위 토큰 사용량 기록
- 시간/일/주 단위 집계 및 모델/계정별 통계
- UI TokenStats 화면과 연동

## 16) 보안 DB (modules/security_db)

- **IP 접근 로그 저장**
- 블랙리스트/화이트리스트 관리
- 요청 통계 및 순위 계산

## 17) Tauri Commands 요약

프론트엔드가 호출하는 대표 명령들:

- 계정: `list_accounts`, `add_account`, `delete_account(s)`, `switch_account`, `fetch_account_quota`, `refresh_all_quotas`
- OAuth: `prepare_oauth_url`, `start_oauth_login`, `complete_oauth_login`, `submit_oauth_code`
- 장치: `get_device_profiles`, `bind_device_profile`, `apply_device_profile`, `restore_original_device`
- 설정: `load_config`, `save_config`, `set_window_theme`
- 업데이트: `check_for_updates`, `get_update_settings`, `save_update_settings`
- 프록시 제어: `start_proxy_service`, `stop_proxy_service`, `get_proxy_status`, `reload_proxy_accounts`
- 로그/통계: `get_proxy_logs`, `get_token_stats_*`
- 보안: `get_ip_access_logs`, `add_ip_to_blacklist`, `get_security_config`

이 명령들이 프론트엔드 모든 기능의 기반이 됩니다.
