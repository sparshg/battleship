use std::ops::{Deref, DerefMut};

use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Board(pub [[char; 10]; 10]);

impl From<Board> for Vec<String> {
    fn from(board: Board) -> Self {
        board.iter().map(|row| row.iter().collect()).collect()
    }
}

impl From<Vec<String>> for Board {
    fn from(board: Vec<String>) -> Self {
        let mut arr = [['e'; 10]; 10];
        for (i, row) in board.iter().enumerate() {
            for (j, cell) in row.chars().enumerate() {
                arr[i][j] = cell;
            }
        }
        Board(arr)
    }
}

impl Deref for Board {
    type Target = [[char; 10]; 10];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
                    board[tx as usize][ty as usize] = 's';
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
                if !(0..10).contains(&tx) || !(0..10).contains(&ty) {
                    continue;
                }
                if self[tx as usize][ty as usize] != 'e' {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_sunk(&self, (i, j): (usize, usize)) -> Option<[(usize, usize); 2]> {
        let mut queue = vec![(i, j)];
        let mut visited = vec![vec![false; 10]; 10];
        let mut bounds = [(i, j), (i, j)];
        visited[i][j] = true;
        while let Some((x, y)) = queue.pop() {
            if self[x][y] == 's' {
                return None;
            }
            bounds[0].0 = bounds[0].0.min(x);
            bounds[0].1 = bounds[0].1.min(y);
            bounds[1].0 = bounds[1].0.max(x);
            bounds[1].1 = bounds[1].1.max(y);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
                let (tx, ty) = ((x as i32 + dx) as usize, (y as i32 + dy) as usize);
                if (0..10).contains(&tx)
                    && (0..10).contains(&ty)
                    && !visited[tx][ty]
                    && matches!(self[tx][ty], 'h' | 's')
                {
                    visited[tx][ty] = true;
                    queue.push((tx, ty));
                }
            }
        }
        Some(bounds)
    }

    pub fn mark_redundant(mut self) -> Self {
        for i in 0..10 {
            for j in 0..10 {
                if self[i][j] == 'h' {
                    for (dx, dy) in [(-1, -1), (1, 1), (1, -1), (-1, 1)].iter() {
                        let (tx, ty) = ((i as i32 + dx) as usize, (j as i32 + dy) as usize);
                        if (0..10).contains(&tx) && (0..10).contains(&ty) {
                            self[tx][ty] = 'm';
                        }
                    }
                    if self.has_sunk((i, j)).is_some() {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
                            let (tx, ty) = ((i as i32 + dx) as usize, (j as i32 + dy) as usize);
                            if (0..10).contains(&tx) && (0..10).contains(&ty) && self[tx][ty] == 'e'
                            {
                                self[tx][ty] = 'm';
                            }
                        }
                    }
                }
            }
        }
        self
    }

    // fn validate_syntax(&self) -> bool {
    //     self
    //         .iter()
    //         .all(|row| row.iter().all(|cell| matches!(cell, 'e' | 'h' | 'm' | 's')))
    // }
}

// pub async fn create_board_route(board: Json<Board>) -> Json<String> {
//     let board = Board::from_json(board).await;
//     Json(format!("{:?}", board))
// }
