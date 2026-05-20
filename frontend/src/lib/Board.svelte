<script>
  import { KIND_NAMES } from './pieces.js';

  export let board;             // 10x9
  export let selected = null;   // {row, col} | null
  export let legalTargets = []; // [{row, col}]
  export let checkedKing = null;// {row, col} | null — 장군 상태의 궁
  export let onCellClick = (r, c) => {};

  function isLegal(r, c) {
    return legalTargets.some(m => m.row === r && m.col === c);
  }
  function isCapture(r, c) {
    return isLegal(r, c) && board[r][c] != null;
  }
  function isChecked(r, c) {
    return checkedKing && checkedKing.row === r && checkedKing.col === c;
  }

  const ROWS = 10;
  const COLS = 9;
  function inHanPalace(r, c) { return r <= 2 && c >= 3 && c <= 5; }
  function inChoPalace(r, c) { return r >= 7 && c >= 3 && c <= 5; }

  // 왕 위치 추적 — 매 board 변경마다 다시 계산
  $: choKing = findKingPos('cho');
  $: hanKing = findKingPos('han');
  function findKingPos(side) {
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        const p = board[r][c];
        if (p && p.side === side && p.kind === 'K') return { row: r, col: c };
      }
    }
    return null;
  }
</script>

<div class="wrap">
  <div class="board-frame">
    <div class="grid" style="--rows: {ROWS}; --cols: {COLS}">
    {#each Array(ROWS * COLS) as _, idx}
      {@const r = Math.floor(idx / COLS)}
      {@const c = idx % COLS}
      {@const p = board[r][c]}
      <button
        class="cell"
        class:han-palace={inHanPalace(r, c)}
        class:cho-palace={inChoPalace(r, c)}
        class:king-cell={p && p.kind === 'K'}
        class:selected={selected && selected.row === r && selected.col === c}
        class:legal={isLegal(r, c) && !isCapture(r, c)}
        class:capture={isCapture(r, c)}
        class:checked={isChecked(r, c)}
        on:click={() => onCellClick(r, c)}
        aria-label={`${r},${c}`}
      >
        {#if p}
          <span class="piece {p.side} kind-{p.kind}">{KIND_NAMES[p.kind][p.side]}</span>
        {/if}
        {#if isLegal(r, c) && !isCapture(r, c)}
          <span class="legal-dot"></span>
        {/if}
      </button>
    {/each}
    </div>
    <!-- 왕 위치 marker — 2x2 cell 진한 사각형 + X 대각선 -->
    <svg class="palace-overlay" viewBox="0 0 9 10" preserveAspectRatio="none" aria-hidden="true">
      {#if hanKing}
        <rect class="king-rect" x={hanKing.col - 0.5} y={hanKing.row - 0.5} width="2" height="2" />
        <line class="king-x" x1={hanKing.col - 0.5} y1={hanKing.row - 0.5} x2={hanKing.col + 1.5} y2={hanKing.row + 1.5} />
        <line class="king-x" x1={hanKing.col + 1.5} y1={hanKing.row - 0.5} x2={hanKing.col - 0.5} y2={hanKing.row + 1.5} />
      {/if}
      {#if choKing}
        <rect class="king-rect" x={choKing.col - 0.5} y={choKing.row - 0.5} width="2" height="2" />
        <line class="king-x" x1={choKing.col - 0.5} y1={choKing.row - 0.5} x2={choKing.col + 1.5} y2={choKing.row + 1.5} />
        <line class="king-x" x1={choKing.col + 1.5} y1={choKing.row - 0.5} x2={choKing.col - 0.5} y2={choKing.row + 1.5} />
      {/if}
    </svg>
  </div>
</div>

<style>
  .wrap {
    display: flex;
    justify-content: center;
    padding: 12px;
  }
  .board-frame {
    position: relative;
    aspect-ratio: 9 / 10;
    width: min(90vmin, 540px);
    background: #d9b277;
    border: 3px solid #3a2912;
    box-shadow: 0 6px 20px rgba(0,0,0,0.3);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), 1fr);
    width: 100%;
    height: 100%;
  }
  .palace-overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 1;
  }
  .palace-overlay .king-rect {
    fill: none;
    stroke: #1a1a1a;
    stroke-width: 0.04;
  }
  .palace-overlay .king-x {
    stroke: #1a1a1a;
    stroke-width: 0.03;
  }
  .cell {
    position: relative;
    padding: 0;
    margin: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
    aspect-ratio: 1 / 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  /* 가로/세로 선 (격자) */
  .cell::before, .cell::after {
    content: "";
    position: absolute;
    background: #3a2912;
  }
  .cell::before { left: 0; right: 0; top: 50%; height: 1px; }
  .cell::after  { top: 0; bottom: 0; left: 50%; width: 1px; }
  /* 가장자리 칸의 선은 반만 — 격자가 보드 안쪽으로만 보이게 */
  .cell:nth-child(9n+1)::before { left: 50%; }
  .cell:nth-child(9n)::before { right: 50%; }
  .cell:nth-child(-n+9)::after { top: 50%; }
  .cell:nth-child(n+82)::after { bottom: 50%; }

  /* 강 (중원) 배경 살짝 */
  .cell:nth-child(n+37):nth-child(-n+45) { background-color: rgba(120, 160, 200, 0.08); }
  .cell:nth-child(n+46):nth-child(-n+54) { background-color: rgba(120, 160, 200, 0.08); }

  /* 궁성 — 대각선 표시 */
  .han-palace.center {}
  .cho-palace.center {}

  /* 왕 cell 자체는 별도 강조 안 함 — SVG palace-overlay의 2x2 사각형으로 표시 */
  .selected {
    background-color: rgba(255, 220, 60, 0.4) !important;
    outline: 2px solid #ffcc33;
    outline-offset: -2px;
  }
  .capture {
    background-color: rgba(255, 60, 60, 0.25) !important;
  }
  .capture .piece {
    outline: 3px solid #ff4d4d;
    outline-offset: 2px;
    box-shadow: 0 0 12px rgba(255, 80, 80, 0.8);
    animation: capture-pulse 0.9s infinite;
  }
  @keyframes capture-pulse {
    0%, 100% { box-shadow: 0 0 10px rgba(255, 80, 80, 0.7); }
    50% { box-shadow: 0 0 18px rgba(255, 80, 80, 1); }
  }
  .checked .piece {
    animation: check-pulse 0.8s infinite;
    outline: 3px solid #ffd54a;
    outline-offset: -2px;
    border-radius: 50%;
  }
  @keyframes check-pulse {
    0%, 100% { box-shadow: 0 0 8px 2px rgba(255, 213, 74, 0.8); }
    50% { box-shadow: 0 0 16px 4px rgba(255, 213, 74, 1); }
  }
  .legal-dot {
    position: absolute;
    width: 36%;
    height: 36%;
    border-radius: 50%;
    background: rgba(80, 220, 120, 0.85);
    z-index: 3;
    pointer-events: none;
    box-shadow: 0 0 10px rgba(80, 220, 120, 0.7);
  }
  .legal:hover { background-color: rgba(80, 220, 120, 0.15) !important; }
  .capture:hover { background-color: rgba(255, 80, 80, 0.4) !important; }
  .cell:hover .piece { transform: scale(1.08); }

  .piece {
    position: relative;
    z-index: 2;
    width: 82%;
    height: 82%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7em;
    font-weight: 800;
    font-family: 'Noto Serif KR', '맑은 고딕', serif;
    transition: transform 0.15s;
    clip-path: polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%);
    filter: drop-shadow(0 2px 3px rgba(0,0,0,0.5));
  }
  /* 종류별 크기 — 궁이 가장 크고 졸이 가장 작음 */
  .piece.kind-K { width: 95%; height: 95%; font-size: 0.92em; }
  .piece.kind-R, .piece.kind-C, .piece.kind-H, .piece.kind-E { width: 82%; height: 82%; font-size: 0.78em; }
  .piece.kind-A { width: 76%; height: 76%; font-size: 0.7em; }
  .piece.kind-P { width: 66%; height: 66%; font-size: 0.58em; }
  .piece.cho {
    background: linear-gradient(160deg, #5fb0ff 0%, #2563d4 50%, #0d3aa0 100%);
    color: white;
  }
  .piece.han {
    background: linear-gradient(160deg, #ff9090 0%, #d83838 50%, #8a0a0a 100%);
    color: white;
  }
  @media (min-width: 500px) {
    .piece.kind-K { font-size: 1.7em; }
    .piece.kind-R, .piece.kind-C, .piece.kind-H, .piece.kind-E { font-size: 1.45em; }
    .piece.kind-A { font-size: 1.3em; }
    .piece.kind-P { font-size: 1.1em; }
  }
</style>
