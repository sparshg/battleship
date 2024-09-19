use socketioxide::socket::Sid;
use thiserror::Error;

use crate::board::Board;

pub const ROOM_CODE_LENGTH: usize = 4;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Room full")]
    RoomFull,
    #[error("Room not full")]
    RoomNotFull,
    #[error("Already in room")]
    AlreadyInRoom,
    #[error("Not in room")]
    NotInRoom,
    #[error("Invalid Move")]
    InvalidMove,
    #[error("SQL Error\n{0:?}")]
    Sqlx(#[from] sqlx::Error),
}

pub async fn add_room(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!(
        r"WITH new_user AS (INSERT INTO players (id, room_code) VALUES ($1, $2) RETURNING id) INSERT INTO rooms (player1_id, code) SELECT $1, $2 FROM new_user",
        sid.as_str(),
        code
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn join_room(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<()> {
    let room = sqlx::query!(
        r#"SELECT player1_id, player2_id FROM rooms WHERE code = $1"#,
        code
    )
    .fetch_one(pool)
    .await?;

    let sid = sid.as_str();

    if room.player1_id.is_some() && room.player2_id.is_some() {
        return Err(Error::RoomFull);
    }
    if let Some(id) = room.player1_id.as_ref() {
        if id == sid {
            return Err(Error::AlreadyInRoom);
        }
    }
    if let Some(id) = room.player2_id.as_ref() {
        if id == sid {
            return Err(Error::AlreadyInRoom);
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

pub async fn add_board(sid: Sid, board: Board, pool: &sqlx::PgPool) -> Result<()> {
    let board: Vec<String> = board.into();
    sqlx::query!(
        "UPDATE players SET board = $1 WHERE id = $2",
        &board,
        sid.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn start(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<()> {
    let room = sqlx::query!(
        r"SELECT player1_id, player2_id FROM rooms WHERE code = $1",
        code
    )
    .fetch_one(pool)
    .await?;

    let (Some(player1), Some(player2)) = (room.player1_id, room.player2_id) else {
        return Err(Error::RoomNotFull); // room not full
    };

    let status = if sid.as_str() == player1 {
        Status::P2Turn
    } else if sid.as_str() == player2 {
        Status::P1Turn
    } else {
        return Err(Error::NotInRoom); // not in room
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
) -> Result<(bool, Option<[(usize, usize); 2]>)> {
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
        _ => return Err(Error::RoomNotFull), // room not full
    };

    let mut board: Board = sqlx::query!(r"SELECT board FROM players WHERE id = $1", other)
        .fetch_one(pool)
        .await?
        .board
        .unwrap()
        .into();

    let hit = match board[i][j] {
        's' => true,
        'e' => false,
        _ => return Err(Error::InvalidMove),
    };
    board[i][j] = if hit { 'h' } else { 'm' };

    let mut txn = pool.begin().await?;
    sqlx::query!(
        r#"UPDATE players SET board[$1] = $2 WHERE id = $3"#,
        i as i32 + 1,
        board[i].iter().collect::<String>(),
        other
    )
    .execute(&mut *txn)
    .await?;

    if !hit {
        sqlx::query!(
            r#"UPDATE rooms SET stat = $1 WHERE code = $2"#,
            to_status as Status,
            player.room_code
        )
        .execute(&mut *txn)
        .await?;
    }

    txn.commit().await?;
    Ok((hit, if hit { board.has_sunk((i, j)) } else { None }))
}

pub async fn disconnect(sid: Sid, pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!(r"DELETE FROM players WHERE id = $1", sid.as_str())
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, sqlx::Type, PartialEq)]
#[sqlx(type_name = "STAT", rename_all = "lowercase")]
enum Status {
    Waiting,
    P1Turn,
    P2Turn,
}
