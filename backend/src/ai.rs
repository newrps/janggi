// 장기 AI — alpha-beta minimax + 기물 가치 평가.

use crate::game::{
    all_legal_moves, simulate, Board, Kind, Move, Piece, Side, COLS, ROWS,
};

fn piece_value(k: Kind) -> i32 {
    match k {
        Kind::K => 100_000,
        Kind::R => 130,
        Kind::C => 70,
        Kind::H => 50,
        Kind::E => 30,
        Kind::A => 30,
        Kind::P => 20,
    }
}

fn evaluate(board: &Board, my_side: Side) -> i32 {
    let mut score = 0i32;
    for r in 0..ROWS {
        for c in 0..COLS {
            if let Some(Piece { side, kind }) = board[r][c] {
                let v = piece_value(kind);
                if side == my_side {
                    score += v;
                } else {
                    score -= v;
                }
            }
        }
    }
    score
}

fn minimax(
    board: &Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    maximizing: bool,
    ai: Side,
) -> i32 {
    if depth == 0 {
        return evaluate(board, ai);
    }
    let side = if maximizing { ai } else { ai.opp() };
    let moves = all_legal_moves(board, side);
    if moves.is_empty() {
        // 합법 이동 없음 = 그 측이 짐
        return if maximizing { -99_000 - depth as i32 } else { 99_000 + depth as i32 };
    }
    if maximizing {
        let mut best = i32::MIN;
        for &(fr, fc, tr, tc) in &moves {
            let sim = simulate(board, fr, fc, tr, tc);
            let score = minimax(&sim, depth - 1, alpha, beta, false, ai);
            if score > best { best = score; }
            if best > alpha { alpha = best; }
            if alpha >= beta { break; }
        }
        best
    } else {
        let mut best = i32::MAX;
        let mut beta_local = beta;
        for &(fr, fc, tr, tc) in &moves {
            let sim = simulate(board, fr, fc, tr, tc);
            let score = minimax(&sim, depth - 1, alpha, beta_local, true, ai);
            if score < best { best = score; }
            if best < beta_local { beta_local = best; }
            if alpha >= beta_local { break; }
        }
        best
    }
}

// 이동 정렬: 적 기물 잡는 수 먼저 (alpha-beta pruning 효과 증가)
fn sorted_moves(board: &Board, side: Side) -> Vec<Move> {
    let mut moves = all_legal_moves(board, side);
    moves.sort_by_key(|&(_, _, tr, tc)| match board[tr][tc] {
        Some(p) => -piece_value(p.kind),
        None => 0,
    });
    moves
}

pub fn best_move(board: &Board, side: Side, depth: u32) -> Option<Move> {
    let moves = sorted_moves(board, side);
    if moves.is_empty() {
        return None;
    }
    let mut best_score = i32::MIN;
    let mut best = moves[0];
    let mut alpha = i32::MIN;
    let beta = i32::MAX;
    for &mv in &moves {
        let (fr, fc, tr, tc) = mv;
        let sim = simulate(board, fr, fc, tr, tc);
        let score = minimax(&sim, depth.saturating_sub(1), alpha, beta, false, side);
        if score > best_score {
            best_score = score;
            best = mv;
        }
        if score > alpha {
            alpha = score;
        }
    }
    Some(best)
}
