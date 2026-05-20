// 장기 온라인 방 매칭 + 실시간 동기화.
// 호스트 = 초(선), 게스트 = 한(후).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::game::{
    all_legal_moves, board_to_json, game_status_after_move, initial_board, is_in_check,
    legal_moves, simulate, Board, GameStatus, PieceJson, Side,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Create,
    Join { code: String },
    Move { from_row: usize, from_col: usize, to_row: usize, to_col: usize },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Created {
        code: String,
        you: String,
        board: Vec<Vec<Option<PieceJson>>>,
    },
    Joined {
        code: String,
        you: String,
        board: Vec<Vec<Option<PieceJson>>>,
    },
    OpponentJoined,
    Move {
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        by: String,
        status: String,
        in_check: bool,
    },
    OpponentLeft,
    Error { msg: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Host,
    Guest,
}

impl Role {
    pub fn side(self) -> Side {
        match self {
            Role::Host => Side::Cho,
            Role::Guest => Side::Han,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Role::Host => "cho",
            Role::Guest => "han",
        }
    }
}

pub struct Room {
    pub code: String,
    pub host: Option<mpsc::UnboundedSender<ServerMsg>>,
    pub guest: Option<mpsc::UnboundedSender<ServerMsg>>,
    pub board: Board,
    pub turn: Side,
    pub status: GameStatus,
}

impl Room {
    fn new(code: String, host: mpsc::UnboundedSender<ServerMsg>) -> Self {
        Self {
            code,
            host: Some(host),
            guest: None,
            board: initial_board(),
            turn: Side::Cho,
            status: GameStatus::Playing,
        }
    }
    fn other_sender(&self, role: Role) -> Option<&mpsc::UnboundedSender<ServerMsg>> {
        match role {
            Role::Host => self.guest.as_ref(),
            Role::Guest => self.host.as_ref(),
        }
    }
}

pub type RoomStore = Arc<Mutex<HashMap<String, Room>>>;

pub fn new_store() -> RoomStore {
    Arc::new(Mutex::new(HashMap::new()))
}

const CODE_ALPHABET: [char; 30] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7',
];

fn gen_code() -> String {
    nanoid::nanoid!(6, &CODE_ALPHABET)
}

fn status_str(s: GameStatus) -> &'static str {
    match s {
        GameStatus::Playing => "playing",
        GameStatus::ChoWin => "cho_win",
        GameStatus::HanWin => "han_win",
    }
}

pub fn handle_create(
    store: &RoomStore,
    my_tx: &mpsc::UnboundedSender<ServerMsg>,
) -> Option<(String, Role)> {
    let mut rooms = store.lock().unwrap();
    let mut code = gen_code();
    while rooms.contains_key(&code) {
        code = gen_code();
    }
    let room = Room::new(code.clone(), my_tx.clone());
    let board_json = board_to_json(&room.board);
    rooms.insert(code.clone(), room);
    let _ = my_tx.send(ServerMsg::Created {
        code: code.clone(),
        you: "cho".into(),
        board: board_json,
    });
    Some((code, Role::Host))
}

pub fn handle_join(
    store: &RoomStore,
    my_tx: &mpsc::UnboundedSender<ServerMsg>,
    code: String,
) -> Option<(String, Role)> {
    let code_upper = code.trim().to_uppercase();
    let mut rooms = store.lock().unwrap();
    let room = match rooms.get_mut(&code_upper) {
        Some(r) => r,
        None => {
            let _ = my_tx.send(ServerMsg::Error { msg: "방을 찾을 수 없어요".into() });
            return None;
        }
    };
    if room.guest.is_some() {
        let _ = my_tx.send(ServerMsg::Error { msg: "이미 가득 찬 방이에요".into() });
        return None;
    }
    if room.host.is_none() {
        let _ = my_tx.send(ServerMsg::Error { msg: "호스트가 나간 방이에요".into() });
        return None;
    }
    room.guest = Some(my_tx.clone());
    let board_json = board_to_json(&room.board);
    let _ = my_tx.send(ServerMsg::Joined {
        code: code_upper.clone(),
        you: "han".into(),
        board: board_json,
    });
    if let Some(h) = &room.host {
        let _ = h.send(ServerMsg::OpponentJoined);
    }
    Some((code_upper, Role::Guest))
}

pub fn handle_move(
    store: &RoomStore,
    my_tx: &mpsc::UnboundedSender<ServerMsg>,
    room_code: &str,
    role: Role,
    fr: usize,
    fc: usize,
    tr: usize,
    tc: usize,
) {
    let mut rooms = store.lock().unwrap();
    let room = match rooms.get_mut(room_code) {
        Some(r) => r,
        None => {
            let _ = my_tx.send(ServerMsg::Error { msg: "방이 없어요".into() });
            return;
        }
    };
    if room.status != GameStatus::Playing {
        let _ = my_tx.send(ServerMsg::Error { msg: "이미 끝난 게임이에요".into() });
        return;
    }
    if room.guest.is_none() {
        let _ = my_tx.send(ServerMsg::Error { msg: "상대를 기다리는 중이에요".into() });
        return;
    }
    let side = role.side();
    if room.turn != side {
        let _ = my_tx.send(ServerMsg::Error { msg: "내 차례가 아니에요".into() });
        return;
    }
    // 룰 검증
    let legal = legal_moves(&room.board, fr, fc);
    if !legal.iter().any(|&(r, c)| r == tr && c == tc) {
        let _ = my_tx.send(ServerMsg::Error { msg: "잘못된 이동이에요".into() });
        return;
    }
    room.board = simulate(&room.board, fr, fc, tr, tc);
    let next = side.opp();
    room.status = game_status_after_move(&room.board, next);
    room.turn = if room.status == GameStatus::Playing { next } else { room.turn };
    let in_check = is_in_check(&room.board, next);
    let move_msg = ServerMsg::Move {
        from_row: fr, from_col: fc, to_row: tr, to_col: tc,
        by: role.name().into(),
        status: status_str(room.status).into(),
        in_check,
    };
    let _ = my_tx.send(move_msg.clone());
    if let Some(other) = room.other_sender(role) {
        let _ = other.send(move_msg);
    }
}

pub fn handle_disconnect(store: &RoomStore, room_code: &str, role: Role) {
    let mut rooms = store.lock().unwrap();
    let room = match rooms.get_mut(room_code) {
        Some(r) => r,
        None => return,
    };
    let other = match role {
        Role::Host => { room.host = None; room.guest.clone() }
        Role::Guest => { room.guest = None; room.host.clone() }
    };
    if let Some(tx) = other {
        let _ = tx.send(ServerMsg::OpponentLeft);
    }
    if room.host.is_none() && room.guest.is_none() {
        rooms.remove(room_code);
    }
}
