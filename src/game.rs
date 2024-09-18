use std::{collections::HashMap, sync::Arc};

use axum::Json;
use rand::Rng;
use serde::Deserialize;
use socketioxide::socket::Sid;
use tokio::sync::RwLock;

pub const ROOM_CODE_LENGTH: usize = 4;

// #[derive(Default, Clone)]
// pub struct Store {
//     rooms: Arc<RwLock<HashMap<String, Room>>>,
//     sockets: Arc<RwLock<HashMap<Sid, String>>>,
// }

// impl Store {
//     pub async fn add_room(&self, code: String) {
//         let mut store = self.rooms.write().await;
//         store.insert(
//             code.clone(),
//             Room {
//                 code,
//                 ..Default::default()
//             },
//         );
//     }
pub async fn add_room(code: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT INTO rooms (code) VALUES ($1)", code)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn join_room(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let room = sqlx::query!(
        r#"SELECT player1_id, player2_id FROM rooms WHERE code = $1"#,
        code
    )
    .fetch_one(pool)
    .await?;

    if room.player1_id.is_some() && room.player2_id.is_some() {
        return Err(sqlx::Error::RowNotFound); // room full
    }

    let mut txn = pool.begin().await?;

    sqlx::query!(
        r#"INSERT INTO players (id, room_code) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET room_code = $2"#,
        sid.as_str(),
        code
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query(&format!(
        "UPDATE rooms SET player{}_id = $1 WHERE code = $2",
        if room.player1_id.is_none() { "1" } else { "2" }
    ))
    .bind(sid.as_str())
    .bind(code)
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;
    Ok(())
}
//     pub async fn join_room(&self, sid: Sid, code: String) -> Result<(), ()> {
//         if self.rooms.read().await.get(&code).is_none() {
//             return Err(());
//         };
//         let mut sockets = self.sockets.write().await;
//         let player = sockets
//             .entry(sid)
//             .and_modify(|p| p.room = Some(code.clone()))
//             .or_insert(Arc::new(Player {
//                 sid,
//                 room: Some(code.clone()),
//                 board: None,
//             }));
//         let mut rooms = self.rooms.write().await;
//         let Some(room) = rooms.get_mut(&code) else {
//             return Err(());
//         };

//         if room.player1.is_none() {
//             room.player1 = Some(Arc::clone(player));
//         }
//         Ok(())
//     }

//     pub async fn add_board(&self, sid: Sid, board: Board) -> Result<(), ()> {
//         let mut store = self.sockets.write().await;
//         if let Some(player) = store.get_mut(&sid) {
//             player.board = Some(board);
//         } else {
//             return Err(());
//         }
//         Ok(())
//     }

//     pub async fn start(&self, code: String, sid: Sid) -> Result<(), ()> {
//         let mut store = self.rooms.write().await;
//         let Some(room) = store.get_mut(&code) else {
//             return Err(());
//         };
//         dbg!(&room);
//         let (Some(player1), Some(player2)) = (room.player1, room.player2) else {
//             return Err(());
//         };

//         if player1.sid == sid {
//             room.status = Status::Player1Turn;
//         } else if player2.sid == sid {
//             room.status = Status::Player2Turn;
//         } else {
//             return Err(());
//         }
//         Ok(())
//     }

//     pub async fn attack(&self, sid: Sid, (i, j): (usize, usize)) -> Result<bool, ()> {
//         let sockets = self.sockets.read().await;
//         let Some(player) = sockets.get(&sid) else {
//             return Err(());
//         };
//         let mut rooms = self.rooms.write().await;
//         let Some(room) = rooms.get_mut(player.room.as_ref().unwrap()) else {
//             return Err(());
//         };

//         match room.status {
//             Status::Player1Turn if player.sid == room.player1.as_ref().unwrap().sid => {
//                 room.status = Status::Player2Turn;
//                 return Ok(room.player2.as_ref().unwrap().board.as_ref().unwrap().0[i][j] == 's');
//             }
//             Status::Player2Turn if player.sid == room.player2.as_ref().unwrap().sid => {
//                 room.status = Status::Player1Turn;
//                 return Ok(room.player1.as_ref().unwrap().board.as_ref().unwrap().0[i][j] == 's');
//             }
//             _ => return Err(()),
//         }

//         Err(())
//     }
// }

// #[derive(Default, Debug)]
// struct Room {
//     code: String,
//     player1: Option<Arc<Player>>,
//     player2: Option<Arc<Player>>,
//     status: Status,
// }

// #[derive(Debug)]
// struct Player {
//     sid: Sid,
//     board: Option<Board>,
//     room: Option<String>,
// }

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "STAT", rename_all = "lowercase")]
enum Status {
    Waiting,
    P1Turn,
    P2Turn,
}

// impl Default for Status {
//     fn default() -> Self {
//         Status::Waiting
//     }
// }

#[derive(Debug, Deserialize)]
pub struct Board(pub [[char; 10]; 10]);

impl Board {
    const SHIPS: [i32; 5] = [5, 4, 3, 3, 2];

    pub fn from_json(Json(board): Json<Board>) -> Self {
        board
    }

    pub fn randomize() -> Self {
        let mut board = Board([['e'; 10]; 10]);
        for &length in Self::SHIPS.iter() {
            loop {
                let dir = rand::thread_rng().gen_bool(0.5);
                let x = rand::thread_rng().gen_range(0..(if dir { 10 } else { 11 - length }));
                let y = rand::thread_rng().gen_range(0..(if dir { 11 - length } else { 10 }));
                if board.is_overlapping(x, y, length, dir) {
                    continue;
                }
                for i in 0..length {
                    let (tx, ty) = if dir { (x, y + i) } else { (x + i, y) };
                    board.0[tx as usize][ty as usize] = 's';
                }
                break;
            }
        }
        board
    }

    fn is_overlapping(&self, x: i32, y: i32, length: i32, dir: bool) -> bool {
        for i in -1..2 {
            for j in -1..=length {
                let (tx, ty) = if dir { (x + i, y + j) } else { (x + j, y + i) };
                if tx < 0 || tx >= 10 || ty < 0 || ty >= 10 {
                    continue;
                }
                if self.0[tx as usize][ty as usize] != 'e' {
                    return true;
                }
            }
        }
        false
    }

    // fn validate_syntax(&self) -> bool {
    //     self.0
    //         .iter()
    //         .all(|row| row.iter().all(|cell| matches!(cell, 'e' | 'h' | 'm' | 's')))
    // }
}

// pub async fn create_board_route(board: Json<Board>) -> Json<String> {
//     let board = Board::from_json(board).await;
//     Json(format!("{:?}", board.0))
// }
