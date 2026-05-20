<script>
  import Board from '$lib/Board.svelte';
  import OnlineGame from '$lib/OnlineGame.svelte';
  import { initialBoard } from '$lib/pieces.js';
  import {
    legalMoves, isLegalMove, applyMove,
    isInCheck, isCheckmate, isStalemate, findKing
  } from '$lib/rules.js';
  import { aiMove } from '$lib/api.js';
  import { playPiece } from '$lib/sound.js';

  let mode = 'menu';     // menu | local | ai | online
  let aiSide = 'han';    // AI는 한(후), 사람이 초(선)가 기본
  let aiDifficulty = 2;
  let aiThinking = false;

  let board = initialBoard();
  let turn = 'cho';
  let selected = null;
  let legalTargets = [];
  let winner = null;
  let winReason = '';
  let inCheck = false;
  let msg = '';

  function reset() {
    board = initialBoard();
    turn = 'cho';
    selected = null;
    legalTargets = [];
    winner = null;
    winReason = '';
    inCheck = false;
    msg = '';
    aiThinking = false;
  }
  function startLocal() { reset(); mode = 'local'; }
  function startAi() {
    reset();
    mode = 'ai';
    if (aiSide === 'cho') {
      // AI가 초(선) → 첫 수
      requestAiMove();
    }
  }
  function backToMenu() { mode = 'menu'; }

  function selectCell(r, c) {
    const p = board[r][c];
    if (p && p.side === turn) {
      selected = { row: r, col: c };
      legalTargets = legalMoves(board, r, c);
      msg = '';
    } else {
      selected = null;
      legalTargets = [];
    }
  }

  async function onCell(r, c) {
    if (winner || aiThinking) return;
    if (mode === 'ai' && turn === aiSide) return;

    if (!selected) { selectCell(r, c); return; }
    if (selected.row === r && selected.col === c) {
      selected = null; legalTargets = []; return;
    }
    const target = board[r][c];
    if (target && target.side === turn) { selectCell(r, c); return; }

    if (!isLegalMove(board, selected.row, selected.col, r, c)) {
      msg = '이 기물은 거기로 갈 수 없어요';
      return;
    }
    const res = applyMove(board, selected.row, selected.col, r, c);
    board = res.board;
    selected = null;
    legalTargets = [];
    playPiece();

    if (res.winner) {
      winner = res.winner; winReason = 'capture'; inCheck = false; return;
    }
    const next = turn === 'cho' ? 'han' : 'cho';
    if (isCheckmate(board, next)) {
      winner = turn; winReason = 'checkmate'; inCheck = true; turn = next; return;
    }
    if (isStalemate(board, next)) {
      winner = turn; winReason = 'stalemate'; inCheck = false; turn = next; return;
    }
    turn = next;
    inCheck = isInCheck(board, next);
    msg = '';

    if (mode === 'ai' && turn === aiSide) {
      await requestAiMove();
    }
  }

  async function requestAiMove() {
    if (winner) return;
    aiThinking = true;
    msg = '';
    try {
      const r = await aiMove(board, aiSide, aiDifficulty);
      board = r.board;
      playPiece();
      if (r.status === 'cho_win' || r.status === 'han_win') {
        winner = r.status === 'cho_win' ? 'cho' : 'han';
        winReason = 'checkmate';
        inCheck = r.in_check;
      } else {
        turn = aiSide === 'cho' ? 'han' : 'cho';
        inCheck = r.in_check;
      }
    } catch (e) {
      msg = `AI 응답 실패: ${e.message}`;
    } finally {
      aiThinking = false;
    }
  }

  $: checkedKing = inCheck ? findKing(board, turn) : null;

  $: statusLabel = (() => {
    if (winner) {
      const name = winner === 'cho' ? '🔵 초(楚)' : '🔴 한(漢)';
      const reason = winReason === 'checkmate' ? '외통수' :
                     winReason === 'stalemate' ? '갈 곳 없음' : '궁 잡힘';
      return `${name} 승리! (${reason})`;
    }
    if (aiThinking) return '🤖 AI 생각중...';
    const turnName = turn === 'cho' ? '🔵 초(楚) 차례' : '🔴 한(漢) 차례';
    return inCheck ? `${turnName} — ⚠ 장군!` : turnName;
  })();
</script>

<main>
  <h1>장기</h1>

  {#if mode === 'menu'}
    <div class="menu">
      <button class="big" on:click={startLocal}>👥 로컬 2인</button>

      <div class="ai-setup">
        <h3>🤖 AI 대전</h3>
        <label>
          내 색:
          <select bind:value={aiSide}>
            <option value="han">🔵 초(楚, 선)</option>
            <option value="cho">🔴 한(漢, 후)</option>
          </select>
        </label>
        <label>
          난이도:
          <select bind:value={aiDifficulty}>
            <option value={1}>쉬움 (depth 2)</option>
            <option value={2}>보통 (depth 3)</option>
            <option value={3}>어려움 (depth 4)</option>
          </select>
        </label>
        <button class="big" on:click={startAi}>AI와 대전 시작</button>
      </div>

      <button class="big online" on:click={() => mode = 'online'}>🌐 온라인 방 매칭</button>
    </div>
  {:else if mode === 'online'}
    <OnlineGame onExit={backToMenu} />
  {:else}
    <div class="status">
      <span class="badge">{mode === 'local' ? '👥 로컬' : `🤖 AI (${['','쉬움','보통','어려움'][aiDifficulty]})`}</span>
      <span class="turn {winner || turn}">{statusLabel}</span>
    </div>

    {#if msg}<div class="msg">{msg}</div>{/if}

    <Board {board} {selected} {legalTargets} {checkedKing} onCellClick={onCell} />

    <div class="actions">
      <button on:click={mode === 'local' ? startLocal : startAi}>↻ 다시</button>
      <button on:click={backToMenu}>← 메뉴</button>
    </div>
  {/if}
</main>

<style>
  :global(body) { margin: 0; background: #1a1a1a; color: #eee; font-family: -apple-system, 'Segoe UI', sans-serif; }
  main { max-width: 760px; margin: 0 auto; padding: 24px 16px; }
  h1 { text-align: center; margin: 0 0 24px; font-size: 32px; letter-spacing: 0.05em; }
  .menu { display: flex; flex-direction: column; gap: 16px; max-width: 360px; margin: 40px auto 0; }
  .ai-setup { background: #2a2a2a; border: 1px solid #444; border-radius: 12px; padding: 20px; display: flex; flex-direction: column; gap: 12px; }
  .ai-setup h3 { margin: 0; font-size: 18px; }
  .ai-setup label { display: flex; justify-content: space-between; align-items: center; font-size: 14px; }
  select { background: #1a1a1a; color: #eee; border: 1px solid #555; padding: 6px 10px; border-radius: 6px; }
  button { background: #3a3a3a; color: #eee; border: 1px solid #555; padding: 10px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; }
  button:hover { background: #4a4a4a; }
  .big { padding: 14px 20px; font-size: 16px; background: #4a6cf7; border-color: #4a6cf7; }
  .big:hover { background: #5a7cff; }
  .big.online { background: #59c275; border-color: #59c275; }
  .big.online:hover { background: #6dd685; }
  .status { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; padding: 0 8px; }
  .badge { background: #2a2a2a; border: 1px solid #444; padding: 4px 10px; border-radius: 6px; font-size: 13px; }
  .turn { font-size: 18px; font-weight: 600; }
  .turn.cho { color: #4ba3ff; }
  .turn.han { color: #ff7a7a; }
  .msg { text-align: center; color: #ffb86b; font-size: 13px; padding: 8px; margin-bottom: 8px; background: rgba(255, 184, 107, 0.1); border-radius: 6px; }
  .actions { display: flex; justify-content: center; gap: 12px; margin-top: 20px; }
</style>
