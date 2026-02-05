# 05. 데이터 저장 구조

Antigravity Tools는 모든 데이터/설정을 로컬에 저장합니다. 이 문서는 파일/DB 위치와 목적을 정리합니다.

## 1) 기본 데이터 디렉토리

- **기본 경로**: `~/.antigravity_tools/`
- 모든 계정 정보, 설정, 로그 DB가 이 디렉토리에 저장됩니다.

## 2) 계정 관련 파일

### 계정 인덱스

- `~/.antigravity_tools/accounts.json`
  - 계정 목록 요약(현재 계정 ID 포함)
  - UI에 표시되는 리스트 정보와 연동

### 계정 상세

- `~/.antigravity_tools/accounts/<account_id>.json`
  - 토큰 정보, 쿼터 정보, 비활성화 상태, 장치 프로필 바인딩 상태 포함

## 3) 설정 파일

- `~/.antigravity_tools/gui_config.json`
  - 전체 App 설정(언어/테마/프록시/라우팅/보안 등)
  - 프록시의 API 키 및 관리자 비밀번호가 이 파일에 저장됨
- `~/.antigravity_tools/http_api_settings.json`
  - 외부 연동용 HTTP API 서버 활성화/포트 설정

## 4) 워밍업 기록

- `~/.antigravity_tools/warmup_history.json`
  - 모델 워밍업 이력과 쿨다운 체크용 기록

## 5) 로그/통계 데이터베이스

### 프록시 요청 로그 DB

- `~/.antigravity_tools/proxy_logs.db`
- 요청 로그 및 상태, 모델/계정/토큰 정보 기록

### 토큰 통계 DB

- `~/.antigravity_tools/token_stats.db`
- 계정별/모델별 토큰 사용량 집계

### 보안 DB

- `~/.antigravity_tools/security.db`
- IP 접근 로그, 블랙리스트/화이트리스트, IP 통계

## 6) 애플리케이션 로그

- `~/.antigravity_tools/logs/app.log*`
  - Tauri/Rust 런타임 로그가 날짜별 파일로 저장됨
  - 오래된 로그는 자동 정리(일수/용량 기준)

## 7) 기타

- Codex 계정 저장소는 `~/.antigravity_tools/codex/accounts.json`에 관리됨
- 장치 지문 관련 데이터는 `device_original.json` 및 계정별 히스토리에 저장됨
- 시스템 캐시는 OS별 경로(예: macOS `~/Library/Caches/...`)에 저장되며, UI에서 일괄 정리 가능

## 8) 운영/복구 가이드

- 계정 복구: `accounts.json`과 `accounts/*.json`을 함께 백업해야 안전합니다.
- 설정 복구: `gui_config.json`을 복원하면 프록시/라우팅 설정이 동일하게 복구됩니다.
- 로그/통계 초기화: DB 파일 삭제 또는 UI의 로그/통계 클리어 기능 사용.
