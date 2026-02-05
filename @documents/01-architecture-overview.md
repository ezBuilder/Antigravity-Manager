# 01. 아키텍처/실행 흐름 개요

## 1) 제품 구성 개요

Antigravity Tools는 **React(Tauri WebView) + Rust(Tauri + Axum) + 로컬 데이터 저장소**로 구성됩니다.

- **프론트엔드**: React + Vite, Zustand 상태 관리, Tauri invoke로 Rust 명령 호출.
- **백엔드(Tauri)**: 계정/설정/장치/업데이트/스케줄러/보안/통계/프록시 제어를 수행.
- **프록시 서버(Axum)**: OpenAI/Claude/Gemini 프로토콜을 제공, 모델 매핑/라우팅/계정 풀/로그/보안 미들웨어를 수행.

## 2) 실행 모드

### A. 데스크톱 모드 (일반 실행)

1. **Tauri 앱 초기화**
   - 로거, 토큰 통계 DB, 보안 DB를 초기화합니다.
2. **Tauri 플러그인 초기화**
   - dialog/fs/opener/autostart/updater/process/window_state/single_instance 등 플러그인을 등록합니다.
3. **관리 서버 기동 + 프록시 자동 실행**
   - 관리 API 및 Web UI 접근을 위해 Axum 서버(기본 8045)를 먼저 열고, 설정의 auto_start 여부에 따라 프록시를 자동 실행합니다.
4. **스마트 스케줄러 기동**
   - 정해진 간격으로 워밍업/계정 상태를 점검합니다.

### B. Headless 모드 (Docker/서버)

1. `--headless` 인자로 실행하면 UI 없이 서버 모드로 진입합니다.
2. 환경변수로 API 키/관리자 비밀번호/인증 모드와 서버 동작을 주입합니다.
   - `ABV_API_KEY` / `API_KEY`: 프록시 API 키 주입
   - `ABV_WEB_PASSWORD` / `WEB_PASSWORD`: 관리자 로그인 비밀번호 주입
   - `ABV_AUTH_MODE` / `AUTH_MODE`: 인증 모드 지정(off/strict/all_except_health/auto)
   - `ABV_MAX_BODY_SIZE`: 요청 바디 최대 크기(기본 100MB)
   - `ABV_DIST_PATH`: Headless 정적 UI 경로 지정
3. Axum 프록시 서버만 기동하며, 스케줄러도 백그라운드에서 동작합니다.

## 3) 컴포넌트 간 연결 구조

```
React UI (Vite)
  └─ Tauri invoke(Commands)
      ├─ modules/* (계정, 설정, 장치, 스케줄러, DB 등)
      └─ proxy/* (Axum 서버, 토큰 매니저, 라우팅, 미들웨어)

Axum Server
  ├─ AI Proxy Routes (/v1/... /v1beta/...)
  ├─ Admin API Routes (/api/...)
  └─ Static Web UI Hosting (Headless/Docker)
```

## 4) 핵심 포인트

- **UI와 백엔드 분리**: React는 상태/화면, Rust는 데이터 및 프로세스 제어.
- **프록시 중심 구조**: 계정 풀과 라우터/미들웨어/로그/보안이 Axum에서 통합 동작.
- **로컬 데이터 중심**: 계정/설정/로그/통계가 모두 로컬 파일/DB로 저장됩니다.

## 5) 문서 연계

- 프론트엔드 상세 기능: **02-frontend-features.md**
- 백엔드 명령/모듈: **03-backend-core.md**
- 프록시/라우팅/보안: **04-proxy-service.md**
- 데이터 저장/스키마: **05-data-storage.md**
