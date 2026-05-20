<script>
  import Board from './Board.svelte';
  import { initialBoard } from './pieces.js';
  import { legalMoves } from './rules.js';
  import { playPiece } from './sound.js';
  import { onDestroy } from 'svelte';

  export let onExit = () => {};

  let phase = 'lobby';
  let mySide = null;     // 'cho' | 'han'
  let roomCode = '';
  let inputCode = '';
  let board = initialBoard();
  let turn = 'cho';
  let selected = null;
  let legalTargets = [];
  let status = 'playing';
  let inCheck = false;
  let checkedKing = null;
  let errMsg = '';
  let ws = null;

  $: if (inCheck) {
    // find king of current turn side
    for (let r = 0; r < 10; r++) {
      for (let c = 0; c < 9; c++) {
        if (board[r][c] && board[r][c].side === turn && board[r][c].kind === 'K') {
          checkedKing = { row: r, col: c };
        }
      }
    }
  } else {
    checkedKing = null;
  }

  function connect() {
    return new Promise((resolve, reject) => {
      const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const sock = new WebSocket(`${proto}//${location.host}/ws/room`);
      sock.onopen = () => { ws = sock; resolve(); };
      sock.onerror = () => reject(new Error('연결 실패'));
      sock.onmessage = (e) => { try { handle(JSON.parse(e.data)); } catch (_) {} };
      sock.onclose = () => {
        if (phase !== 'lobby' && phase !== 'left' && phase !== 'ended') {
          errMsg = '서버 연결이 끊겼어요';
          phase = 'left';
        }
      };
    });
  }

  function send(msg) {
    if (ws && ws.readyState === 1) ws.send(JSON.stringify(msg));
  }

  function handle(msg) {
    errMsg = '';
    switch (msg.type) {
      case 'created':
        roomCode = msg.code;
        mySide = msg.you;
        board = msg.board;
        turn = 'cho';
        status = 'playing';
        inCheck = false;
        phase = 'wait';
        break;
      case 'joined':
        roomCode = msg.code;
        mySide = msg.you;
        board = msg.board;
        turn = 'cho';
        status = 'playing';
        inCheck = false;
        phase = 'playing';
        break;
      case 'opponent_joined':
        phase = 'playing';
        break;
      case 'move': {
        // 서버가 검증된 이동을 알려줌 — 로컬 보드 갱신
        const moving = board[msg.from_row][msg.from_col];
        board[msg.from_row][msg.from_col] = null;
        board[msg.to_row] = [...board[msg.to_row]];
        board[msg.to_row][msg.to_col] = moving;
        board = board;
        status = msg.status;
        inCheck = msg.in_check;
        turn = msg.by === 'cho' ? 'han' : 'cho';
        selected = null;
        legalTargets = [];
        playPiece();
        if (status !== 'playing') phase = 'ended';
        break;
      }
      case 'opponent_left':
        errMsg = '상대가 나갔어요';
        phase = 'left';
        break;
      case 'error':
        errMsg = msg.msg;
        break;
    }
  }

  async function createRoom() {
    errMsg = '';
    try { await connect(); send({ type: 'create' }); }
    catch (e) { errMsg = e.message; }
  }
  async function joinRoom() {
    errMsg = '';
    const code = inputCode.trim().toUpperCase();
    if (!code) { errMsg = '방 코드를 입력해주세요'; return; }
    try { await connect(); send({ type: 'join', code }); }
    catch (e) { errMsg = e.message; }
  }

  function onCell(r, c) {
    if (phase !== 'playing') return;
    if (turn !== mySide) return;

    if (!selected) {
      const p = board[r][c];
      if (p && p.side === mySide) {
        selected = { row: r, col: c };
        legalTargets = legalMoves(board, r, c);
      }
      return;
    }
    if (selected.row === r && selected.col === c) {
      selected = null; legalTargets = []; return;
    }
    const target = board[r][c];
    if (target && target.side === mySide) {
      selected = { row: r, col: c };
      legalTargets = legalMoves(board, r, c);
      return;
    }
    // 서버에 이동 요청 (검증은 서버가)
    send({
      type: 'move',
      from_row: selected.row, from_col: selected.col,
      to_row: r, to_col: c
    });
  }

  function backToLobby() {
    if (ws) { ws.close(); ws = null; }
    phase = 'lobby';
    mySide = null; roomCode = ''; inputCode = '';
    board = initialBoard(); turn = 'cho'; selected = null; legalTargets = [];
    status = 'playing'; inCheck = false; checkedKing = null;
    errMsg = '';
  }

  function copyCode() { navigator.clipboard?.writeText(roomCode); }

  onDestroy(() => { if (ws) ws.close(); });

  $: statusLabel = (() => {
    if (status === 'cho_win') return '🔵 초(楚) 승리!';
    if (status === 'han_win') return '🔴 한(漢) 승리!';
    if (phase === 'wait') return '⏳ 상대 기다리는 중...';
    if (phase === 'left') return '🚪 게임 종료';
    const base = turn === mySide ? '✅ 내 차례' : '⏳ 상대 차례';
    return inCheck ? `${base} — ⚠ 장군!` : base;
  })();
</script>

<div class="wrap">
  <div class="top">
    <button class="back" on:click={onExit}>← 메뉴</button>
    <span class="title">🌐 온라인</span>
  </div>

  {#if phase === 'lobby'}
    <div class="lobby">
      <div class="card">
        <h3>방 만들기 (초)</h3>
        <p class="hint">방 만들면 코드 받음, 친구에게 공유</p>
        <button class="big" on:click={createRoom}>방 만들기</button>
      </div>
      <div class="card">
        <h3>방 참가 (한)</h3>
        <p class="hint">6자리 코드 입력</p>
        <input type="text" maxlength="6" placeholder="ABCD23"
               bind:value={inputCode}
               on:keydown={(e) => e.key === 'Enter' && joinRoom()} />
        <button class="big" on:click={joinRoom}>참가</button>
      </div>
      {#if errMsg}<div class="err">{errMsg}</div>{/if}
    </div>
  {:else}
    <div class="status">
      <div class="room-info">
        <span class="badge">{mySide === 'cho' ? '🔵 초' : '🔴 한'}</span>
        <button class="code" on:click={copyCode} title="복사">{roomCode}</button>
      </div>
      <span class="turn">{statusLabel}</span>
    </div>

    {#if errMsg}<div class="err">{errMsg}</div>{/if}

    <Board {board} {selected} {legalTargets} {checkedKing} onCellClick={onCell} />

    {#if phase === 'ended' || phase === 'left'}
      <div class="actions">
        <button on:click={backToLobby}>새 방 / 다른 방</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .wrap { max-width: 760px; margin: 0 auto; }
  .top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
  .back, button { background: #3a3a3a; color: #eee; border: 1px solid #555; padding: 8px 14px; border-radius: 6px; cursor: pointer; font-size: 14px; }
  button:hover { background: #4a4a4a; }
  .title { font-size: 18px; font-weight: 600; }
  .lobby { display: flex; flex-direction: column; gap: 16px; max-width: 360px; margin: 40px auto 0; }
  .card { background: #2a2a2a; border: 1px solid #444; border-radius: 12px; padding: 20px; display: flex; flex-direction: column; gap: 10px; }
  .card h3 { margin: 0; font-size: 18px; }
  .hint { margin: 0; color: #888; font-size: 13px; }
  input { background: #1a1a1a; color: #eee; border: 1px solid #555; padding: 10px 14px; border-radius: 6px; font-size: 18px; font-family: ui-monospace, monospace; text-transform: uppercase; letter-spacing: 0.1em; text-align: center; }
  .big { padding: 12px 20px; font-size: 16px; background: #4a6cf7; border-color: #4a6cf7; }
  .big:hover { background: #5a7cff; }
  .status { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; padding: 0 8px; }
  .room-info { display: flex; gap: 8px; align-items: center; }
  .badge { background: #2a2a2a; border: 1px solid #444; padding: 4px 10px; border-radius: 6px; font-size: 13px; }
  .code { background: #4a6cf7; border-color: #4a6cf7; color: white; font-family: ui-monospace, monospace; letter-spacing: 0.1em; font-weight: 600; padding: 4px 10px; font-size: 14px; }
  .turn { font-size: 18px; font-weight: 600; }
  .actions { display: flex; justify-content: center; gap: 12px; margin-top: 20px; }
  .err { background: #4a1e1e; border: 1px solid #8a3a3a; padding: 8px 12px; border-radius: 6px; margin-bottom: 12px; font-size: 13px; }
</style>
