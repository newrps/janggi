// 장기 이동 룰. 클라이언트 사이드 검증 (Phase 3에서 백엔드 검증 추가 예정).
//
// 보드: 10 row × 9 col. row 0~2=한 궁성, row 7~9=초 궁성.
// 궁성 대각선 라인 위 좌표는 차/포/궁/사가 대각선 이동 가능.

const ROWS = 10;
const COLS = 9;

function inBoard(r, c) {
  return r >= 0 && r < ROWS && c >= 0 && c < COLS;
}

function inPalace(r, c, side) {
  if (c < 3 || c > 5) return false;
  if (side === 'han') return r >= 0 && r <= 2;
  return r >= 7 && r <= 9;
}

// 양 궁성의 대각선 라인 위 좌표 — 중앙 + 네 모서리
function isPalaceDiagLine(r, c) {
  if (r === 1 && c === 4) return true;       // 한 궁성 중앙
  if (r === 8 && c === 4) return true;       // 초 궁성 중앙
  if ((r === 0 || r === 2) && (c === 3 || c === 5)) return true; // 한 모서리
  if ((r === 7 || r === 9) && (c === 3 || c === 5)) return true; // 초 모서리
  return false;
}

function isEnemy(cell, side) {
  return cell && cell.side !== side;
}
function isFriend(cell, side) {
  return cell && cell.side === side;
}

// ---- 기물별 이동 ----

// 궁/사: 궁성 안에서 1칸 (가/세). 대각선은 궁성 diag line 위에서만.
function kingGuardMoves(board, r, c, side) {
  const moves = [];
  for (const [dr, dc] of [[-1,0],[1,0],[0,-1],[0,1]]) {
    const nr = r + dr, nc = c + dc;
    if (!inPalace(nr, nc, side)) continue;
    if (isFriend(board[nr][nc], side)) continue;
    moves.push({ row: nr, col: nc });
  }
  if (isPalaceDiagLine(r, c)) {
    for (const [dr, dc] of [[-1,-1],[-1,1],[1,-1],[1,1]]) {
      const nr = r + dr, nc = c + dc;
      if (!inPalace(nr, nc, side)) continue;
      if (!isPalaceDiagLine(nr, nc)) continue;
      if (isFriend(board[nr][nc], side)) continue;
      moves.push({ row: nr, col: nc });
    }
  }
  return moves;
}

// 차: 직선 무제한. 같은 사이드 막힘, 적 잡고 정지. 궁성 대각선도 동일.
function chariotMoves(board, r, c, side) {
  const moves = [];
  const dirs = [[-1,0],[1,0],[0,-1],[0,1]];
  for (const [dr, dc] of dirs) {
    let nr = r + dr, nc = c + dc;
    while (inBoard(nr, nc)) {
      const t = board[nr][nc];
      if (!t) moves.push({ row: nr, col: nc });
      else {
        if (t.side !== side) moves.push({ row: nr, col: nc });
        break;
      }
      nr += dr; nc += dc;
    }
  }
  if (isPalaceDiagLine(r, c)) {
    for (const [dr, dc] of [[-1,-1],[-1,1],[1,-1],[1,1]]) {
      let nr = r + dr, nc = c + dc;
      while (inBoard(nr, nc) && isPalaceDiagLine(nr, nc)) {
        const t = board[nr][nc];
        if (!t) moves.push({ row: nr, col: nc });
        else {
          if (t.side !== side) moves.push({ row: nr, col: nc });
          break;
        }
        nr += dr; nc += dc;
      }
    }
  }
  return moves;
}

// 포: 직선으로 다른 기물 1개 점프. 점프 대상이 포면 X, 도착지가 포면 못 잡음.
function cannonMoves(board, r, c, side) {
  const moves = [];
  const scan = (dr, dc, requireDiag) => {
    let nr = r + dr, nc = c + dc;
    let jumped = false;
    while (inBoard(nr, nc) && (!requireDiag || isPalaceDiagLine(nr, nc))) {
      const t = board[nr][nc];
      if (!jumped) {
        // 점프 대상 찾는 중
        if (t) {
          if (t.kind === 'C') return; // 포는 점프 불가
          jumped = true;
        }
      } else {
        // 점프 후 도착 가능 칸 찾는 중
        if (!t) {
          moves.push({ row: nr, col: nc });
        } else {
          if (t.kind !== 'C' && t.side !== side) {
            moves.push({ row: nr, col: nc });
          }
          return;
        }
      }
      nr += dr; nc += dc;
    }
  };
  for (const [dr, dc] of [[-1,0],[1,0],[0,-1],[0,1]]) scan(dr, dc, false);
  if (isPalaceDiagLine(r, c)) {
    for (const [dr, dc] of [[-1,-1],[-1,1],[1,-1],[1,1]]) scan(dr, dc, true);
  }
  return moves;
}

// 마: 직선 1칸 + 그 방향에서 대각선 1칸 더 (체스 나이트와 동일한 도착 패턴, 단 다리 막힘 룰).
// 변위: (±2, ±1) 또는 (±1, ±2).
function horseMoves(board, r, c, side) {
  const moves = [];
  // (다리dr, 다리dc, 최종dr, 최종dc) — 시작점 기준 변위
  const patterns = [
    [-1, 0, -2, -1], [-1, 0, -2, 1],   // 위 1 → 좌상/우상 (최종 위로 2, 좌/우 1)
    [ 1, 0,  2, -1], [ 1, 0,  2, 1],   // 아래
    [ 0,-1, -1, -2], [ 0,-1,  1,-2],   // 좌 1 → 좌상/좌하
    [ 0, 1, -1,  2], [ 0, 1,  1, 2],   // 우
  ];
  for (const [d1r, d1c, d2r, d2c] of patterns) {
    const br = r + d1r, bc = c + d1c;
    if (!inBoard(br, bc) || board[br][bc]) continue;  // 다리 막힘
    const tr = r + d2r, tc = c + d2c;
    if (!inBoard(tr, tc)) continue;
    if (isFriend(board[tr][tc], side)) continue;
    moves.push({ row: tr, col: tc });
  }
  return moves;
}

// 상: 직선 1칸 + 대각선 2칸. 변위: (±3, ±2) 또는 (±2, ±3). 경로 막히면 X.
function elephantMoves(board, r, c, side) {
  const moves = [];
  // (다리1dr,dc, 다리2dr,dc, 최종dr,dc)
  const patterns = [
    [-1, 0, -2, -1, -3, -2], [-1, 0, -2, 1, -3, 2],  // 위 1 → 대각 2칸
    [ 1, 0,  2, -1,  3, -2], [ 1, 0,  2, 1,  3, 2],
    [ 0,-1, -1, -2, -2, -3], [ 0,-1,  1,-2,  2,-3],  // 좌 1 → 대각 2칸
    [ 0, 1, -1,  2, -2,  3], [ 0, 1,  1, 2,  2, 3],
  ];
  for (const [d1r, d1c, d2r, d2c, d3r, d3c] of patterns) {
    const r1 = r + d1r, c1 = c + d1c;
    if (!inBoard(r1, c1) || board[r1][c1]) continue;
    const r2 = r + d2r, c2 = c + d2c;
    if (!inBoard(r2, c2) || board[r2][c2]) continue;
    const r3 = r + d3r, c3 = c + d3c;
    if (!inBoard(r3, c3)) continue;
    if (isFriend(board[r3][c3], side)) continue;
    moves.push({ row: r3, col: c3 });
  }
  return moves;
}

// 졸/병: 1칸 전진 또는 좌/우 1칸. 후진 X.
// 적 궁성 안에서는 대각선 전진도 가능 (한국 장기 표준)
function soldierMoves(board, r, c, side) {
  const moves = [];
  const forward = side === 'cho' ? -1 : 1; // 초는 위로(-1), 한은 아래(+1)
  const dirs = [[forward, 0], [0, -1], [0, 1]];
  for (const [dr, dc] of dirs) {
    const nr = r + dr, nc = c + dc;
    if (!inBoard(nr, nc)) continue;
    if (isFriend(board[nr][nc], side)) continue;
    moves.push({ row: nr, col: nc });
  }
  // 적 궁성 안 (또는 진입 가능한 위치)에서 대각선 전진
  // 졸이 적 궁성 dia 라인 위에 있으면 대각선 1칸 전진 가능
  const enemySide = side === 'cho' ? 'han' : 'cho';
  if (inPalace(r, c, enemySide) && isPalaceDiagLine(r, c)) {
    for (const dc of [-1, 1]) {
      const nr = r + forward, nc = c + dc;
      if (!inBoard(nr, nc)) continue;
      if (!inPalace(nr, nc, enemySide)) continue;
      if (!isPalaceDiagLine(nr, nc)) continue;
      if (isFriend(board[nr][nc], side)) continue;
      moves.push({ row: nr, col: nc });
    }
  }
  return moves;
}

// 기물 자체 이동 룰만 검증 (self-check 고려 X). 내부용.
function pseudoMoves(board, r, c) {
  const p = board[r][c];
  if (!p) return [];
  switch (p.kind) {
    case 'K':
    case 'A': return kingGuardMoves(board, r, c, p.side);
    case 'R': return chariotMoves(board, r, c, p.side);
    case 'C': return cannonMoves(board, r, c, p.side);
    case 'H': return horseMoves(board, r, c, p.side);
    case 'E': return elephantMoves(board, r, c, p.side);
    case 'P': return soldierMoves(board, r, c, p.side);
    default: return [];
  }
}

function simulate(board, fromR, fromC, toR, toC) {
  const nb = board.map((row) => row.slice());
  nb[toR][toC] = nb[fromR][fromC];
  nb[fromR][fromC] = null;
  return nb;
}

export function findKing(board, side) {
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const p = board[r][c];
      if (p && p.side === side && p.kind === 'K') return { row: r, col: c };
    }
  }
  return null;
}

// side의 궁이 적의 다음 수에 잡힐 수 있는가
export function isInCheck(board, side) {
  const king = findKing(board, side);
  if (!king) return false;
  const enemy = side === 'cho' ? 'han' : 'cho';
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const p = board[r][c];
      if (p && p.side === enemy) {
        const moves = pseudoMoves(board, r, c);
        if (moves.some((m) => m.row === king.row && m.col === king.col)) {
          return true;
        }
      }
    }
  }
  return false;
}

// 빅장: 양 궁이 같은 열에서 사이가 비어있는 상태
export function isFacingKings(board) {
  const k1 = findKing(board, 'cho');
  const k2 = findKing(board, 'han');
  if (!k1 || !k2 || k1.col !== k2.col) return false;
  const r1 = Math.min(k1.row, k2.row);
  const r2 = Math.max(k1.row, k2.row);
  for (let r = r1 + 1; r < r2; r++) {
    if (board[r][k1.col]) return false;
  }
  return true;
}

// 진짜 합법 이동: pseudo - (자기 궁 잡힘) - (빅장 만들기)
export function legalMoves(board, r, c) {
  const piece = board[r][c];
  if (!piece) return [];
  return pseudoMoves(board, r, c).filter((m) => {
    const sim = simulate(board, r, c, m.row, m.col);
    if (isInCheck(sim, piece.side)) return false;
    if (isFacingKings(sim)) return false;
    return true;
  });
}

export { simulate };

export function isLegalMove(board, fromR, fromC, toR, toC) {
  return legalMoves(board, fromR, fromC).some(
    (m) => m.row === toR && m.col === toC
  );
}

// side가 합법 이동을 하나라도 가지고 있나
export function hasAnyLegalMove(board, side) {
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const p = board[r][c];
      if (p && p.side === side) {
        if (legalMoves(board, r, c).length > 0) return true;
      }
    }
  }
  return false;
}

export function isCheckmate(board, side) {
  return isInCheck(board, side) && !hasAnyLegalMove(board, side);
}

// 스테일메이트(장군 아닌데 둘 수 없음) — 한국 장기에서는 패배 처리
export function isStalemate(board, side) {
  return !isInCheck(board, side) && !hasAnyLegalMove(board, side);
}

// 이동 적용. 결과: { board, captured, winner }
// winner: 'cho' | 'han' | null — 적 궁 잡혔으면 그 적의 반대 사이드가 승.
export function applyMove(board, fromR, fromC, toR, toC) {
  const newBoard = board.map((row) => row.slice());
  const moving = newBoard[fromR][fromC];
  const captured = newBoard[toR][toC];
  newBoard[fromR][fromC] = null;
  newBoard[toR][toC] = moving;
  let winner = null;
  if (captured && captured.kind === 'K') {
    winner = captured.side === 'cho' ? 'han' : 'cho';
  }
  return { board: newBoard, captured, winner };
}
