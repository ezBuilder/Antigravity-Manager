# Antigravity Tools 기능 문서화 (@documents)

이 폴더는 Antigravity Tools(=Antigravity-Manager) 전체 기능을 개발 이전에 파악할 수 있도록 정리한 상세 문서입니다. 문서들은 **“무엇을 제공하는가”**와 **“어떻게 구현되는가(구성 요소, 상태/데이터 흐름, 주요 모듈)”**를 함께 설명합니다. 각 문서는 서로 참조되며, 핵심 모듈/엔드포인트/데이터 저장 구조까지 포함합니다.

## 문서 구성

- **01-architecture-overview.md**
  - 제품 개요, 데스크톱/헤드리스 실행 흐름, 전체 아키텍처, 런타임 구성(React + Tauri + Axum Proxy)을 설명합니다.
- **02-frontend-features.md**
  - UI 페이지별 기능(대시보드/계정/프록시/모니터링/보안/설정/통계)과 상태 관리, 서비스 호출 구조를 설명합니다.
- **03-backend-core.md**
  - Tauri 명령(Commands)과 핵심 모듈(계정/장치/설정/마이그레이션/스케줄러/업데이트/보안 DB/토큰 통계)을 설명합니다.
- **04-proxy-service.md**
  - Axum 기반 프록시 서버 구성, 프로토콜 호환(OpenAI/Claude/Gemini), 모델 라우팅, 토큰 관리자, 보안/모니터링, 관리자 API를 상세히 정리합니다.
- **05-data-storage.md**
  - 로컬 저장소 구조(계정 JSON, 인덱스, DB 파일), 로그/통계/보안 DB 스키마, 설정 파일을 설명합니다.

## 읽는 순서 추천

1. **01-architecture-overview.md**로 전체 개념을 이해한 뒤
2. **02-frontend-features.md**(화면/기능)와 **03-backend-core.md**(서버/로컬 기능)를 병행
3. **04-proxy-service.md**에서 핵심 프록시 동작/라우팅/보안을 심층 파악
4. **05-data-storage.md**로 저장 구조/운영 관점을 보완

## 범위

- **모든 주요 기능**: 계정 관리(OAuth/토큰/배치), 프록시/API 호환, 모델 라우팅/PM Router, 모니터링/로그/통계, 보안(IP 필터/블랙·화이트리스트), 업데이트/자동 실행/CLI 설정 동기화, 장치 지문, 워밍업 스케줄러, Codex 계정 연동 등.
- **구현 디테일**: React + Zustand 상태 흐름, Tauri Command 호출, Rust 모듈 경로, 프록시 라우팅/미들웨어 구성, 로컬 DB 구조.

필요한 기능 확장/변경을 진행할 때 이 문서를 기준으로 **영향 범위**와 **연동 지점**을 빠르게 파악할 수 있도록 구성했습니다.
