# 장기 — 배포 가이드

## 포트 / 도메인

| 항목 | 값 |
|---|---|
| 컨테이너 노출 포트 | `18085` |
| 백엔드 내부 포트 | `8774` |
| 권장 도메인 | `janggi.zam.kr` |

---

## 1. NAS 폴더 생성 (최초 1회)

```bash
ssh -p 56822 newrps@192.168.123.110
mkdir -p /volume1/docker/janggi
exit
```

---

## 2. deploy-all.ps1 CustomMap 등록 (최초 1회)

`C:\git\deploy-all.ps1` 상단 `$CustomMap`에 추가:

```powershell
"ps_jangki" = "janggi"
```

---

## 3. 배포

```powershell
# C:\git 에서 (프로젝트 이름 지정)
.\deploy-all.ps1 ps_jangki

# 또는 이 폴더에서 바로
.\deploy.ps1

# 강제 재빌드
.\deploy.ps1 -Force
```

---

## 4. Synology Reverse Proxy 설정 (최초 1회)

**제어판 → 로그인 포털 → 고급 → 역방향 프록시 → 생성**

| 필드 | 값 |
|---|---|
| 설명 | janggi |
| 원본 프로토콜 | HTTPS |
| 원본 호스트명 | `janggi.zam.kr` |
| 원본 포트 | 443 |
| 대상 프로토콜 | HTTP |
| 대상 호스트명 | localhost |
| 대상 포트 | **18085** |

**사용자 정의 헤더 탭 → 만들기 → WebSocket** 선택  
(온라인 대전 `/ws/` 경로에 WebSocket 필요)

---

## 5. HTTPS 인증서 (최초 1회)

**제어판 → 보안 → 인증서 → 추가 → Let's Encrypt**

- 도메인: `janggi.zam.kr`
- 인증서 설정에서 역방향 프록시 항목에 이 인증서 지정

---

## 6. 구조

```
ps_jangki/
├── backend/          # Rust + Axum (포트 8774)
│   ├── src/
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/         # SvelteKit → nginx
│   ├── src/
│   ├── nginx.conf    # /api/, /ws/ → backend 프록시 포함
│   └── Dockerfile
├── docker-compose.yml
└── deploy.ps1
```

---

## 7. 로컬 개발

```powershell
# 백엔드 (포트 8774)
cd backend
cargo run

# 프론트 (포트 5179)
cd frontend
npm run dev
```

또는 `games` 명령으로 세 게임 한 번에 기동.

---

## 8. 트러블슈팅

| 증상 | 조치 |
|---|---|
| AI 응답 500 | 백엔드 컨테이너 로그 확인: `docker logs ps-jangki-backend` |
| AI가 너무 느림 | 난이도를 낮추거나 백엔드 CPU 사용량 확인 |
| WebSocket 연결 실패 | Reverse Proxy에 WebSocket 헤더 설정 여부 확인 |
| 배포 후 빈 화면 | `docker logs ps-jangki-frontend` 확인 |
| NAS 매칭 실패 | `.\deploy-all.ps1 -ListOnly` 로 매핑 확인 |
