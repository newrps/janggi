mod ai;
mod game;
mod room;

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};

use game::{
    all_legal_moves, board_to_json, game_status_after_move, is_in_check, parse_board, simulate,
    PieceJson, Side,
};
use room::{
    handle_create, handle_disconnect, handle_join, handle_move, new_store, ClientMsg, Role,
    RoomStore, ServerMsg,
};

#[derive(Clone)]
struct AppState {
    rooms: RoomStore,
}

#[derive(Debug, Deserialize)]
struct AiMoveReq {
    board: Vec<Vec<Option<PieceJson>>>,
    ai_side: String,   // "cho" | "han"
    difficulty: u8,    // 1~3
}

#[derive(Debug, Serialize)]
struct AiMoveResp {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
    board: Vec<Vec<Option<PieceJson>>>,
    status: String,
    in_check: bool,
}

fn status_str(s: game::GameStatus) -> &'static str {
    match s {
        game::GameStatus::Playing => "playing",
        game::GameStatus::ChoWin => "cho_win",
        game::GameStatus::HanWin => "han_win",
    }
}

async fn ai_move(Json(req): Json<AiMoveReq>) -> impl IntoResponse {
    let board = match parse_board(&req.board) {
        Ok(b) => b,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let side = match req.ai_side.as_str() {
        "cho" => Side::Cho,
        "han" => Side::Han,
        _ => return (StatusCode::BAD_REQUEST, "ai_side must be cho or han").into_response(),
    };
    let depth = match req.difficulty {
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 2,
    };
    let result = tokio::task::spawn_blocking(move || {
        let mv = ai::best_move(&board, side, depth)?;
        let (fr, fc, tr, tc) = mv;
        let after = simulate(&board, fr, fc, tr, tc);
        let next = side.opp();
        let status = game_status_after_move(&after, next);
        let in_check = is_in_check(&after, next);
        Some((fr, fc, tr, tc, after, status, in_check))
    })
    .await;
    let (fr, fc, tr, tc, after, status, in_check) = match result {
        Ok(Some(v)) => v,
        _ => return (StatusCode::CONFLICT, "no legal move").into_response(),
    };
    Json(AiMoveResp {
        from_row: fr,
        from_col: fc,
        to_row: tr,
        to_col: tc,
        board: board_to_json(&after),
        status: status_str(status).into(),
        in_check,
    })
    .into_response()
}

async fn health() -> &'static str { "ok" }

async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if ws_sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let mut my_room: Option<String> = None;
    let mut my_role: Option<Role> = None;

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let parsed: Result<ClientMsg, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ClientMsg::Create) => {
                        if my_room.is_some() {
                            let _ = tx.send(ServerMsg::Error { msg: "이미 방에 있어요".into() });
                            continue;
                        }
                        if let Some((code, role)) = handle_create(&state.rooms, &tx) {
                            my_room = Some(code);
                            my_role = Some(role);
                        }
                    }
                    Ok(ClientMsg::Join { code }) => {
                        if my_room.is_some() {
                            let _ = tx.send(ServerMsg::Error { msg: "이미 방에 있어요".into() });
                            continue;
                        }
                        if let Some((code, role)) = handle_join(&state.rooms, &tx, code) {
                            my_room = Some(code);
                            my_role = Some(role);
                        }
                    }
                    Ok(ClientMsg::Move { from_row, from_col, to_row, to_col }) => {
                        let (Some(code), Some(role)) = (my_room.as_ref(), my_role) else {
                            let _ = tx.send(ServerMsg::Error { msg: "방에 들어가지 않았어요".into() });
                            continue;
                        };
                        handle_move(&state.rooms, &tx, code, role, from_row, from_col, to_row, to_col);
                    }
                    Err(_) => {
                        let _ = tx.send(ServerMsg::Error { msg: "잘못된 메시지".into() });
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    if let (Some(code), Some(role)) = (my_room, my_role) {
        handle_disconnect(&state.rooms, &code, role);
    }
    send_task.abort();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8774);

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let state = Arc::new(AppState { rooms: new_store() });

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/ai-move", post(ai_move))
        .route("/ws/room", get(ws_handler))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("ps-jangki backend listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
