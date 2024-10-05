use rand::Rng;
use serde::Serialize;
use socketioxide::socket::Sid;
use thiserror::Error;

use crate::board::Board;

pub const ROOM_CODE_LENGTH: usize = 4;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Room full, potential replacement {0:?}")]
    RoomFull(Option<String>),
    #[error("Room not full")]
    RoomNotFull,
    #[error("GameOver room joined")]
    GameOverRoom,
    #[error("Already in room")]
    AlreadyInRoom,
    #[error("Not in room")]
    NotInRoom,
    #[error("Invalid Move")]
    InvalidMove,
    #[error("Code Generation Limit Reached")]
    CodeGenerationLimitReached,
    #[error("SQL Error\n{0:?}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, sqlx::Type, PartialEq, Serialize)]
#[sqlx(type_name = "STAT", rename_all = "lowercase")]
pub enum Status {
    Waiting,
    P1Turn,
    P2Turn,
    GameOver,
}

pub async fn room_if_player_exists(sid: &str, pool: &sqlx::PgPool) -> Result<Option<String>> {
    Ok(
        sqlx::query!("SELECT room_code FROM players WHERE id = $1", sid)
            .fetch_optional(pool)
            .await?
            .map(|player| player.room_code),
    )
}

async fn generate_code(pool: &sqlx::PgPool) -> Result<String> {
    for _ in 0..50 {
        let code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(ROOM_CODE_LENGTH)
            .map(|x| char::to_ascii_uppercase(&(x as char)))
            .collect();
        if sqlx::query!(r"SELECT code FROM rooms WHERE code = $1", code)
            .fetch_optional(pool)
            .await?
            .is_none()
        {
            return Ok(code);
        }
    }
    Err(Error::CodeGenerationLimitReached)
}

pub async fn add_room(sid: Sid, pool: &sqlx::PgPool) -> Result<String> {
    delete_sid(sid.as_str(), pool).await?;
    let code = generate_code(pool).await?;

    sqlx::query!(
        r"WITH new_user AS (INSERT INTO players (id, room_code) VALUES ($1, $2) RETURNING id) INSERT INTO rooms (player1_id, code) SELECT $1, $2 FROM new_user",
        sid.as_str(),
        code
    )
    .execute(pool)
    .await?;
    Ok(code)
}

pub async fn join_room(sid: Sid, code: String, pool: &sqlx::PgPool) -> Result<()> {
    let code = code.to_uppercase();
    let room = sqlx::query!(
        r#"SELECT player1_id, player2_id, stat AS "stat: Status" FROM rooms WHERE code = $1"#,
        code
    )
    .fetch_one(pool)
    .await?;

    let sid = sid.as_str();

    // if player is already in room
    if [room.player1_id.as_ref(), room.player2_id.as_ref()]
        .into_iter()
        .flatten()
        .filter(|&x| x == sid)
        .next()
        .is_some()
    {
        // if game was over, set status to waiting and return
        if room.stat == Status::GameOver {
            sqlx::query!(
                r"UPDATE rooms SET stat = $1 WHERE code = $2",
                Status::Waiting as Status,
                code
            )
            .execute(pool)
            .await?;
            return Ok(());
        }
        return Err(Error::AlreadyInRoom);
    }

    if room.stat == Status::GameOver {
        return Err(Error::GameOverRoom);
    }

    if let (Some(p1), Some(p2)) = (room.player1_id.as_ref(), room.player2_id.as_ref()) {
        if in_delete_sid(p1, pool).await? {
            // update_sid(p1, sid, pool).await?;
            return Err(Error::RoomFull(Some(p1.to_string())));
        } else if in_delete_sid(p2, pool).await? {
            // update_sid(p2, sid, pool).await?;
            return Err(Error::RoomFull(Some(p2.to_string())));
        }
        return Err(Error::RoomFull(None));
    }
    delete_sid(sid, pool).await?;
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

pub async fn get_room(sid: Sid, pool: &sqlx::PgPool) -> Result<Option<String>> {
    Ok(
        sqlx::query!("SELECT room_code FROM players WHERE id = $1", sid.as_str())
            .fetch_optional(pool)
            .await?
            .map(|r| r.room_code),
    )
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

pub async fn get_game_state(
    sid: &str,
    room: &str,
    pool: &sqlx::PgPool,
) -> Result<(bool, Vec<String>, Vec<String>, bool)> {
    let room_details = sqlx::query!(
        r#"SELECT player1_id, player2_id, stat AS "stat: Status" FROM rooms WHERE code = $1"#,
        room
    )
    .fetch_one(pool)
    .await?;

    let turn = match room_details.stat {
        Status::P1Turn if room_details.player1_id == Some(sid.to_string()) => true,
        Status::P2Turn if room_details.player2_id == Some(sid.to_string()) => true,
        _ => false,
    };

    let oid = match (room_details.player1_id, room_details.player2_id) {
        (Some(p1), Some(p2)) if p1 == sid => p2,
        (Some(p1), Some(p2)) if p2 == sid => p1,
        _ => return Err(Error::NotInRoom),
    };

    let player_board: Board = sqlx::query!(
        r#"SELECT board FROM players WHERE id = $1 AND room_code = $2"#,
        sid,
        room
    )
    .fetch_one(pool)
    .await?
    .board
    .unwrap()
    .into();

    let opponent_board: Board = sqlx::query!(
        r#"SELECT board FROM players WHERE id = $1 AND room_code = $2"#,
        oid,
        room
    )
    .fetch_one(pool)
    .await?
    .board
    .unwrap()
    .into();

    let game_over = player_board.is_game_over() || opponent_board.is_game_over();

    let player_board: Vec<String> = player_board.mark_redundant().into();

    let opponent_board: Vec<String> = opponent_board.mark_redundant().into();
    let opponent_board: Vec<String> = opponent_board
        .into_iter()
        .map(|row| {
            row.chars()
                .map(|x| if x == 's' { 'e' } else { x })
                .collect()
        })
        .collect::<Vec<_>>();

    Ok((turn, player_board, opponent_board, game_over))
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
        Status::P1Turn
    } else if sid.as_str() == player2 {
        Status::P2Turn
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
) -> Result<(bool, Option<[(usize, usize); 2]>, bool)> {
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
    let game_over = board.is_game_over();
    if game_over {
        sqlx::query!(
            r#"UPDATE rooms SET stat = $1 WHERE code = $2"#,
            Status::GameOver as Status,
            player.room_code
        )
        .execute(&mut *txn)
        .await?;
    }
    
    txn.commit().await?;
    Ok((hit, if hit { board.has_sunk((i, j)) } else { None }, game_over))
}

pub async fn update_sid(oldsid: &str, newsid: &str, pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!(
        r"UPDATE players SET id = $1, abandoned = FALSE WHERE id = $2",
        newsid,
        oldsid
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_sid(sid: &str, pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!(r"DELETE FROM players WHERE id = $1", sid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn to_delete_sid(sid: &str, pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query!(r"UPDATE players SET abandoned = TRUE WHERE id = $1", sid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn in_delete_sid(sid: &str, pool: &sqlx::PgPool) -> Result<bool> {
    Ok(
        sqlx::query!(r"SELECT abandoned FROM players WHERE id = $1", sid)
            .fetch_one(pool)
            .await?
            .abandoned,
    )
}
