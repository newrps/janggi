// 장기 게임 룰 (Rust 포팅).
// 보드 10×9, 위=한, 아래=초.

use serde::{Deserialize, Serialize};

pub const ROWS: usize = 10;
pub const COLS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Cho,
    Han,
}

impl Side {
    pub fn opp(self) -> Side {
        match self {
            Side::Cho => Side::Han,
            Side::Han => Side::Cho,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    K, A, R, C, H, E, P,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Piece {
    pub side: Side,
    pub kind: Kind,
}

pub type Board = [[Option<Piece>; COLS]; ROWS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Playing,
    ChoWin,
    HanWin,
}

pub fn initial_board() -> Board {
    let mut b: Board = [[None; COLS]; ROWS];
    let han = |k| Some(Piece { side: Side::Han, kind: k });
    let cho = |k| Some(Piece { side: Side::Cho, kind: k });
    // 한측
    b[0][0] = han(Kind::R); b[0][1] = han(Kind::H); b[0][2] = han(Kind::E);
    b[0][3] = han(Kind::A); b[0][5] = han(Kind::A); b[0][6] = han(Kind::E);
    b[0][7] = han(Kind::H); b[0][8] = han(Kind::R);
    b[1][4] = han(Kind::K);
    b[2][1] = han(Kind::C); b[2][7] = han(Kind::C);
    for c in (0..COLS).step_by(2) { b[3][c] = han(Kind::P); }
    // 초측
    for c in (0..COLS).step_by(2) { b[6][c] = cho(Kind::P); }
    b[7][1] = cho(Kind::C); b[7][7] = cho(Kind::C);
    b[8][4] = cho(Kind::K);
    b[9][0] = cho(Kind::R); b[9][1] = cho(Kind::H); b[9][2] = cho(Kind::E);
    b[9][3] = cho(Kind::A); b[9][5] = cho(Kind::A); b[9][6] = cho(Kind::E);
    b[9][7] = cho(Kind::H); b[9][8] = cho(Kind::R);
    b
}

#[inline]
fn in_board(r: i32, c: i32) -> bool {
    r >= 0 && r < ROWS as i32 && c >= 0 && c < COLS as i32
}

fn in_palace(r: i32, c: i32, side: Side) -> bool {
    if c < 3 || c > 5 { return false; }
    match side {
        Side::Han => r >= 0 && r <= 2,
        Side::Cho => r >= 7 && r <= 9,
    }
}

fn is_palace_diag(r: i32, c: i32) -> bool {
    if r == 1 && c == 4 { return true; }
    if r == 8 && c == 4 { return true; }
    if (r == 0 || r == 2) && (c == 3 || c == 5) { return true; }
    if (r == 7 || r == 9) && (c == 3 || c == 5) { return true; }
    false
}

fn cell(board: &Board, r: i32, c: i32) -> Option<Piece> {
    if !in_board(r, c) { return None; }
    board[r as usize][c as usize]
}

fn is_friend(p: Option<Piece>, side: Side) -> bool {
    matches!(p, Some(x) if x.side == side)
}

pub type Move = (usize, usize, usize, usize); // (fr, fc, tr, tc)

// ---- 각 기물별 pseudo-legal moves ----

fn add_king_guard(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
        let nr = r + dr; let nc = c + dc;
        if !in_palace(nr, nc, side) { continue; }
        if is_friend(cell(board, nr, nc), side) { continue; }
        out.push((nr as usize, nc as usize));
    }
    if is_palace_diag(r, c) {
        for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
            let nr = r + dr; let nc = c + dc;
            if !in_palace(nr, nc, side) { continue; }
            if !is_palace_diag(nr, nc) { continue; }
            if is_friend(cell(board, nr, nc), side) { continue; }
            out.push((nr as usize, nc as usize));
        }
    }
}

fn add_chariot(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
        let mut nr = r + dr; let mut nc = c + dc;
        while in_board(nr, nc) {
            let t = cell(board, nr, nc);
            match t {
                None => out.push((nr as usize, nc as usize)),
                Some(p) => {
                    if p.side != side { out.push((nr as usize, nc as usize)); }
                    break;
                }
            }
            nr += dr; nc += dc;
        }
    }
    if is_palace_diag(r, c) {
        for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
            let mut nr = r + dr; let mut nc = c + dc;
            while in_board(nr, nc) && is_palace_diag(nr, nc) {
                let t = cell(board, nr, nc);
                match t {
                    None => out.push((nr as usize, nc as usize)),
                    Some(p) => {
                        if p.side != side { out.push((nr as usize, nc as usize)); }
                        break;
                    }
                }
                nr += dr; nc += dc;
            }
        }
    }
}

fn add_cannon(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    let scan = |dr: i32, dc: i32, require_diag: bool, out: &mut Vec<(usize, usize)>| {
        let mut nr = r + dr; let mut nc = c + dc;
        let mut jumped = false;
        while in_board(nr, nc) && (!require_diag || is_palace_diag(nr, nc)) {
            let t = cell(board, nr, nc);
            if !jumped {
                if let Some(p) = t {
                    if p.kind == Kind::C { return; }
                    jumped = true;
                }
            } else {
                match t {
                    None => out.push((nr as usize, nc as usize)),
                    Some(p) => {
                        if p.kind != Kind::C && p.side != side {
                            out.push((nr as usize, nc as usize));
                        }
                        return;
                    }
                }
            }
            nr += dr; nc += dc;
        }
    };
    for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] { scan(dr, dc, false, out); }
    if is_palace_diag(r, c) {
        for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] { scan(dr, dc, true, out); }
    }
}

fn add_horse(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    // (다리dr, 다리dc, 최종dr, 최종dc) — 시작점 기준 변위
    let patterns: [(i32, i32, i32, i32); 8] = [
        (-1, 0, -2, -1), (-1, 0, -2, 1),
        (1, 0, 2, -1), (1, 0, 2, 1),
        (0, -1, -1, -2), (0, -1, 1, -2),
        (0, 1, -1, 2), (0, 1, 1, 2),
    ];
    for (d1r, d1c, d2r, d2c) in patterns {
        let br = r + d1r; let bc = c + d1c;
        if !in_board(br, bc) { continue; }
        if cell(board, br, bc).is_some() { continue; }
        let tr = r + d2r; let tc = c + d2c;
        if !in_board(tr, tc) { continue; }
        if is_friend(cell(board, tr, tc), side) { continue; }
        out.push((tr as usize, tc as usize));
    }
}

fn add_elephant(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    // (다리1dr,dc, 다리2dr,dc, 최종dr,dc)
    let patterns: [(i32, i32, i32, i32, i32, i32); 8] = [
        (-1, 0, -2, -1, -3, -2), (-1, 0, -2, 1, -3, 2),
        (1, 0, 2, -1, 3, -2), (1, 0, 2, 1, 3, 2),
        (0, -1, -1, -2, -2, -3), (0, -1, 1, -2, 2, -3),
        (0, 1, -1, 2, -2, 3), (0, 1, 1, 2, 2, 3),
    ];
    for (d1r, d1c, d2r, d2c, d3r, d3c) in patterns {
        let r1 = r + d1r; let c1 = c + d1c;
        if !in_board(r1, c1) || cell(board, r1, c1).is_some() { continue; }
        let r2 = r + d2r; let c2 = c + d2c;
        if !in_board(r2, c2) || cell(board, r2, c2).is_some() { continue; }
        let r3 = r + d3r; let c3 = c + d3c;
        if !in_board(r3, c3) { continue; }
        if is_friend(cell(board, r3, c3), side) { continue; }
        out.push((r3 as usize, c3 as usize));
    }
}

fn add_soldier(board: &Board, r: i32, c: i32, side: Side, out: &mut Vec<(usize, usize)>) {
    let forward = if side == Side::Cho { -1 } else { 1 };
    for (dr, dc) in [(forward, 0i32), (0, -1), (0, 1)] {
        let nr = r + dr; let nc = c + dc;
        if !in_board(nr, nc) { continue; }
        if is_friend(cell(board, nr, nc), side) { continue; }
        out.push((nr as usize, nc as usize));
    }
    let enemy = side.opp();
    if in_palace(r, c, enemy) && is_palace_diag(r, c) {
        for dc in [-1i32, 1] {
            let nr = r + forward; let nc = c + dc;
            if !in_board(nr, nc) { continue; }
            if !in_palace(nr, nc, enemy) { continue; }
            if !is_palace_diag(nr, nc) { continue; }
            if is_friend(cell(board, nr, nc), side) { continue; }
            out.push((nr as usize, nc as usize));
        }
    }
}

pub fn pseudo_moves(board: &Board, r: usize, c: usize) -> Vec<(usize, usize)> {
    let p = match board[r][c] { Some(p) => p, None => return vec![] };
    let mut out = Vec::with_capacity(20);
    let (ri, ci) = (r as i32, c as i32);
    match p.kind {
        Kind::K | Kind::A => add_king_guard(board, ri, ci, p.side, &mut out),
        Kind::R => add_chariot(board, ri, ci, p.side, &mut out),
        Kind::C => add_cannon(board, ri, ci, p.side, &mut out),
        Kind::H => add_horse(board, ri, ci, p.side, &mut out),
        Kind::E => add_elephant(board, ri, ci, p.side, &mut out),
        Kind::P => add_soldier(board, ri, ci, p.side, &mut out),
    }
    out
}

pub fn find_king(board: &Board, side: Side) -> Option<(usize, usize)> {
    for r in 0..ROWS {
        for c in 0..COLS {
            if let Some(p) = board[r][c] {
                if p.side == side && p.kind == Kind::K {
                    return Some((r, c));
                }
            }
        }
    }
    None
}

pub fn is_in_check(board: &Board, side: Side) -> bool {
    let (kr, kc) = match find_king(board, side) { Some(k) => k, None => return false };
    let enemy = side.opp();
    for r in 0..ROWS {
        for c in 0..COLS {
            if let Some(p) = board[r][c] {
                if p.side == enemy {
                    for (nr, nc) in pseudo_moves(board, r, c) {
                        if nr == kr && nc == kc { return true; }
                    }
                }
            }
        }
    }
    false
}

pub fn is_facing_kings(board: &Board) -> bool {
    let k1 = match find_king(board, Side::Cho) { Some(k) => k, None => return false };
    let k2 = match find_king(board, Side::Han) { Some(k) => k, None => return false };
    if k1.1 != k2.1 { return false; }
    let (r1, r2) = if k1.0 < k2.0 { (k1.0, k2.0) } else { (k2.0, k1.0) };
    for r in (r1 + 1)..r2 {
        if board[r][k1.1].is_some() { return false; }
    }
    true
}

pub fn simulate(board: &Board, fr: usize, fc: usize, tr: usize, tc: usize) -> Board {
    let mut nb = *board;
    nb[tr][tc] = nb[fr][fc];
    nb[fr][fc] = None;
    nb
}

pub fn legal_moves(board: &Board, r: usize, c: usize) -> Vec<(usize, usize)> {
    let p = match board[r][c] { Some(p) => p, None => return vec![] };
    pseudo_moves(board, r, c)
        .into_iter()
        .filter(|&(tr, tc)| {
            let sim = simulate(board, r, c, tr, tc);
            !is_in_check(&sim, p.side) && !is_facing_kings(&sim)
        })
        .collect()
}

pub fn all_legal_moves(board: &Board, side: Side) -> Vec<Move> {
    let mut out = Vec::new();
    for r in 0..ROWS {
        for c in 0..COLS {
            if let Some(p) = board[r][c] {
                if p.side == side {
                    for (tr, tc) in legal_moves(board, r, c) {
                        out.push((r, c, tr, tc));
                    }
                }
            }
        }
    }
    out
}

pub fn game_status_after_move(board: &Board, next_to_move: Side) -> GameStatus {
    // 다음 차례 사이드가 합법 이동 없으면 (외통/스테일메이트) → 상대 승
    if all_legal_moves(board, next_to_move).is_empty() {
        return match next_to_move {
            Side::Cho => GameStatus::HanWin,
            Side::Han => GameStatus::ChoWin,
        };
    }
    GameStatus::Playing
}

// 보드를 JS와 같은 형태(Vec<Vec<Option<piece-shape>>>)로 받기 위한 helper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PieceJson {
    pub side: String,
    pub kind: String,
}

pub fn parse_board(v: &Vec<Vec<Option<PieceJson>>>) -> Result<Board, &'static str> {
    if v.len() != ROWS { return Err("invalid rows"); }
    let mut b: Board = [[None; COLS]; ROWS];
    for r in 0..ROWS {
        if v[r].len() != COLS { return Err("invalid cols"); }
        for c in 0..COLS {
            if let Some(pj) = &v[r][c] {
                let side = match pj.side.as_str() {
                    "cho" => Side::Cho,
                    "han" => Side::Han,
                    _ => return Err("bad side"),
                };
                let kind = match pj.kind.as_str() {
                    "K" => Kind::K, "A" => Kind::A, "R" => Kind::R, "C" => Kind::C,
                    "H" => Kind::H, "E" => Kind::E, "P" => Kind::P,
                    _ => return Err("bad kind"),
                };
                b[r][c] = Some(Piece { side, kind });
            }
        }
    }
    Ok(b)
}

pub fn board_to_json(board: &Board) -> Vec<Vec<Option<PieceJson>>> {
    let mut out = Vec::with_capacity(ROWS);
    for r in 0..ROWS {
        let mut row = Vec::with_capacity(COLS);
        for c in 0..COLS {
            row.push(board[r][c].map(|p| PieceJson {
                side: match p.side { Side::Cho => "cho".into(), Side::Han => "han".into() },
                kind: match p.kind {
                    Kind::K => "K", Kind::A => "A", Kind::R => "R", Kind::C => "C",
                    Kind::H => "H", Kind::E => "E", Kind::P => "P",
                }.into(),
            }));
        }
        out.push(row);
    }
    out
}
