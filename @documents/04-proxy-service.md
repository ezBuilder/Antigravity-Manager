# 04. 프록시 서비스 상세 (Axum)

프록시는 Antigravity Tools의 핵심 기능으로, OpenAI/Claude/Gemini API를 **단일 서버(8045)**로 통합 제공합니다. 이 문서는 프록시의 구성과 요청 처리 흐름을 설명합니다.

## 1) 구조 개요

- **AxumServer**가 프록시 및 관리자 API를 동시에 제공
- **TokenManager**가 계정 풀/스케줄링/레이트리밋 상태 관리
- **Mappers**가 프로토콜 변환(OpenAI ↔ Claude ↔ Gemini)
- **Middleware**가 인증/보안/모니터링/IP 필터를 담당

## 2) 프로토콜 라우팅 (AI Proxy Routes)

다음 API가 동일 서버에서 동작합니다.

### OpenAI 호환

- `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses` (Codex CLI 호환)
- `/v1/images/generations`, `/v1/images/edits`
- `/v1/audio/transcriptions`

### Claude/Anthropic 호환

- `/v1/messages`
- `/v1/messages/count_tokens`
- `/v1/models/claude`

### Gemini 호환 (Native)

- `/v1beta/models`
- `/v1beta/models/:model` (GET info / POST generateContent)
- `/v1beta/models/:model/countTokens`

### 기타

- `/health`, `/healthz` 헬스체크
- `/internal/warmup` 내부 워밍업
- `/v1/api/event_logging` (클라이언트 노이즈 방지용 OK 응답)
- MCP Web Search/Reader, Z.ai MCP 엔드포인트

## 3) 관리자 API (/api)

관리 API는 **강제 인증**이 적용됩니다.

- 계정: list/add/delete/switch/refresh/import/sync/export/reorder
- 장치 지문: bind/restore/versions
- 프록시: start/stop/status/mapping/api-key/session-bindings/rate-limit
- 통계: token stats(시간/일/주/모델/계정), proxy stats
- 로그: 요청 로그 리스트/상세/삭제
- 보안: IP 로그/블랙·화이트리스트/보안 설정
- 업데이트: 체크/설정/상태 기록
- CLI Sync: Claude/Codex/Gemini 설정 동기화
- Cloudflared: install/start/stop/status

## 3-1) 서버 실행 옵션/환경변수

- `ABV_MAX_BODY_SIZE`: 요청 바디 최대 크기(기본 100MB)
- `ABV_DIST_PATH`: Headless 모드에서 정적 UI 파일 경로 지정
- `ABV_API_KEY` / `API_KEY`: 프록시 API 키 주입
- `ABV_WEB_PASSWORD` / `WEB_PASSWORD`: 관리자 비밀번호 주입
- `ABV_AUTH_MODE` / `AUTH_MODE`: 인증 정책(off/strict/all_except_health/auto)

## 4) 인증/보안 미들웨어

### 4-1. Auth 미들웨어

- Proxy 요청과 Admin 요청을 분리하여 인증 정책을 다르게 적용
- `ProxyAuthMode`에 따라 **Off / Strict / AllExceptHealth / Auto** 처리
- Admin API는 `admin_password` 우선, 없으면 `api_key` fallback

### 4-2. IP 필터 미들웨어

- 블랙리스트/화이트리스트 적용
- `x-forwarded-for`, `x-real-ip`, TCP 연결 IP 순으로 클라이언트 IP 추출
- 차단 시 보안 DB에 로그 기록 및 상세 메시지 반환

### 4-3. 보안 모니터링

- IP 접근 로그를 별도 보안 DB에 기록해 Security 화면에 노출
- 블랙리스트/화이트리스트 정책에 따라 즉시 차단/허용

## 5) TokenManager: 계정 풀/스케줄링

- `accounts/*.json`에서 계정 로드 및 Token 구조 생성
- **레이트리밋/세션 바인딩 관리**
- **Sticky Session(세션 고정) 및 고정 계정(preferred_account_id) 모드 지원**
- Codex 계정(별도 저장소)도 proxy token 풀에 포함

## 6) 모델 매핑/라우팅

- **Custom Mapping**: 요청 모델명을 사용자 정의 모델로 매핑
- **PM Router**: 요청 내용을 분석해 PM-lite/PM-pro 모델로 동적 라우팅
- **Z.ai Provider**: Claude 요청을 z.ai로 라우팅 가능

## 7) 모니터링/로깅

- 요청 로그는 메모리 + SQLite DB에 저장
- 토큰 사용량 기록이 TokenStats DB로 집계됨
- Web UI/Monitor 페이지에서 실시간 확인 가능

## 7-1) 정적 UI 호스팅

- Headless/Docker 환경에서 Axum 서버가 `dist` 또는 `ABV_DIST_PATH`의 정적 파일을 제공

## 8) 디버그/실험 기능

- Debug Logging: 요청/응답 상세 기록
- Experimental 플래그: Signature Cache, Tool Loop Recovery, Context Usage Scaling 등

## 9) 요청 흐름 요약

1. 클라이언트 요청 수신
2. Auth/IP 필터 통과 여부 확인
3. 프로토콜별 handler에서 요청 변환
4. TokenManager로 계정 선택
5. Upstream 요청 → 응답 변환
6. 로그/통계 기록 및 결과 반환
