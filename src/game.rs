use axum::Json;
use rand::Rng;
use serde::Deserialize;
use socketioxide::socket::Sid;

pub const ROOM_CODE_LENGTH: usize = 4;

pub async fn add_room(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"WITH new_user AS (INSERT INTO players (id, room_code) VALUES ($1, $2) RETURNING id) INSERT INTO rooms (player1_id, code) SELECT $1, $2 FROM new_user",
        sid.as_str(),
        code
    )
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

    let sid = sid.as_str();

    if room.player1_id.is_some() && room.player2_id.is_some() {
        return Err(sqlx::Error::RowNotFound); // room full
    }
    if let Some(id) = room.player1_id.as_ref() {
        if id == sid {
            return Err(sqlx::Error::RowNotFound); // already in room
        }
    }
    if let Some(id) = room.player2_id.as_ref() {
        if id == sid {
            return Err(sqlx::Error::RowNotFound); // already in room
        }
    }

    let mut txn = pool.begin().await?;

    // create/update player
    sqlx::query!(
        r#"INSERT INTO players (id, room_code) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET room_code = $2"#,
        sid,
        code
    )
    .execute(&mut *txn)
    .await?;

    // add to room
    sqlx::query(&format!(
        "UPDATE rooms SET player{}_id = $1 WHERE code = $2",
        if room.player1_id.is_none() { "1" } else { "2" }
    ))
    .bind(sid)
    .bind(code)
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;
    Ok(())
}

pub async fn add_board(sid: Sid, board: Board, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let query = format!(
        "UPDATE players SET board = ARRAY[{}] WHERE id = '{}'",
        board
            .0
            .map(|row| {
                format!(
                    "ARRAY[{}]",
                    row.map(|x| format!("'{x}'"))
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .into_iter()
            .collect::<Vec<String>>()
            .join(","),
        sid.as_str()
    );
    sqlx::query(&query).execute(pool).await.unwrap();
    Ok(())
}

pub async fn start(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let room = sqlx::query!(
        r"SELECT player1_id, player2_id FROM rooms WHERE code = $1",
        code
    )
    .fetch_one(pool)
    .await?;

    let (Some(player1), Some(player2)) = (room.player1_id, room.player2_id) else {
        return Err(sqlx::Error::RowNotFound); // room not full
    };

    let status = if sid.as_str() == player1 {
        Status::P2Turn
    } else if sid.as_str() == player2 {
        Status::P1Turn
    } else {
        return Err(sqlx::Error::RowNotFound); // not in room
    };

    sqlx::query!(
        r"UPDATE rooms SET stat = $1 WHERE code = $2",
        status as Status,
        code
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn attack(
    sid: Sid,
    (i, j): (usize, usize),
    pool: &sqlx::PgPool,
) -> Result<bool, sqlx::Error> {
    let player = sqlx::query!(r"SELECT room_code FROM players WHERE id = $1", sid.as_str())
        .fetch_one(pool)
        .await?;

    let room = sqlx::query!(
        r#"SELECT stat AS "stat: Status", player1_id, player2_id FROM rooms WHERE code = $1"#,
        player.room_code
    )
    .fetch_one(pool)
    .await?;

    let (_, other, to_status) = match (room.player1_id, room.player2_id) {
        (Some(p1), Some(p2)) if p1 == sid.as_str() && room.stat == Status::P1Turn => {
            (p1, p2, Status::P2Turn)
        }
        (Some(p1), Some(p2)) if p2 == sid.as_str() && room.stat == Status::P2Turn => {
            (p2, p1, Status::P1Turn)
        }
        _ => return Err(sqlx::Error::RowNotFound), // room not full
    };

    let mut txn = pool.begin().await?;

    let turn = sqlx::query!(
        r"SELECT board[$1][$2] as HIT FROM players WHERE id = $3",
        i as i32 + 1,
        j as i32 + 1,
        other
    )
    .fetch_one(&mut *txn)
    .await?;

    sqlx::query!(
        r#"UPDATE players
        SET board[$1][$2] = CASE
                WHEN board[$1][$2] = 's' THEN 'h'
                WHEN board[$1][$2] = 'e' THEN 'm'
                ELSE board[$1][$2]
            END
        WHERE id = $3"#,
        i as i32 + 1,
        j as i32 + 1,
        other
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"UPDATE rooms SET stat = $1 WHERE code = $2"#,
        to_status as Status,
        player.room_code
    )
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;
    Ok(turn.hit.unwrap() == "s")
}

pub async fn disconnect(sid: Sid, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(r"DELETE FROM players WHERE id = $1", sid.as_str())
        .execute(pool)
        .await
        .unwrap();
    Ok(())
}

#[derive(Debug, sqlx::Type, PartialEq)]
#[sqlx(type_name = "STAT", rename_all = "lowercase")]
enum Status {
    Waiting,
    P1Turn,
    P2Turn,
}

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
