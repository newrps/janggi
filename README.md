# ps_jangki

장기 (Korean Chess) 웹게임.

- **Backend**: Rust + axum 0.7 (Phase 2부터 룰 검증/AI/온라인 추가)
- **Frontend**: SvelteKit + adapter-static (9×10 보드, 한자 기물)
- **Deploy**: docker-compose (backend :8774 / frontend nginx :18085)

## Phase
- ✅ Phase 1: 프로젝트 구조 + 보드 + 초기 배치 + 자유 이동 (룰 검증 없음)
- ⏳ Phase 2: 기물별 이동 룰 + 궁성 제약 + 외통 판정
- ⏳ Phase 3: AI (minimax + iterative deepening)
- ⏳ Phase 4: 온라인 방 매칭

## 룰 메모 (Phase 2 구현 예정)
- **궁/사**: 궁성 안에서 1칸 (대각선은 궁성 대각선만)
- **차**: 직선 무제한
- **포**: 직선으로 다른 기물 1개 점프, 단 점프 대상이 포는 X. 포로 포 못 잡음
- **마**: 1+1 (먼저 가/세 1, 그 다음 대각 1), 첫 칸 막히면 X
- **상**: 1+2 (가/세 1, 그 다음 대각 2번), 경로 막히면 X
- **졸/병**: 1칸 전진. 강 건너면 좌우 1칸 가능
- 승리: 궁 잡으면 승

## 로컬 개발
```bash
# Backend
cd backend && cargo run
# Frontend
cd frontend && npm install && npm run dev
```
브라우저: http://localhost:5179
