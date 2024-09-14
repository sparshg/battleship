use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Board(pub [[char; 10]; 10]);

impl Board {
    const SHIPS: [i32; 5] = [5, 4, 3, 3, 2];

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

    pub async fn from_json(Json(board): Json<Board>) -> Self {
        board
    }

    fn validate_syntax(&self) -> bool {
        self.0
            .iter()
            .all(|row| row.iter().all(|cell| matches!(cell, 'e' | 'h' | 'm' | 's')))
    }
}

pub async fn create_board_route(board: Json<Board>) -> String {
    let board = Board::from_json(board).await;
    format!("{:?}", board)
}
