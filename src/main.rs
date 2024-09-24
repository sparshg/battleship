mod board;
mod game;
use axum::Router;
use board::Board;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use game::{
    add_board, add_room, attack, delete_sid, get_game_state, get_room, join_room,
    room_if_player_exists, start, to_delete_sid, update_sid, Error, ROOM_CODE_LENGTH,
};

use serde::Deserialize;
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;
    let _ = dotenv();
    let url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::postgres::PgPool::connect(&url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    sqlx::query("DELETE FROM players").execute(&pool).await?;
    let (layer, io) = SocketIo::builder().with_state(pool).build_layer();

    io.ns("/", on_connect);
    let app = Router::new()
        // .route("/", post(game::create_board_route))
        .layer(layer)
        .layer(CorsLayer::very_permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    pub session: Option<String>,
}

async fn on_connect(socket: SocketRef, Data(auth): Data<AuthPayload>, pool: State<PgPool>) {
    tracing::info!("Connected: {:?}", socket.id);
    tracing::info!("Connected: {:?}", auth.session);

    if let Some(sid) = auth.session {
        update_sid(&sid, socket.id.as_str(), &pool).await.unwrap();
        let sid = socket.id.as_str();
        if let Some(room) = room_if_player_exists(sid, &pool).await.unwrap() {
            let data = get_game_state(sid, &room, &pool).await.unwrap();
            socket
                .emit(
                    "restore",
                    serde_json::json!({"turn": data.0, "player": data.1, "opponent": data.2}),
                )
                .unwrap();
            socket.join(room.clone()).unwrap();
            emit_update_room(
                &socket,
                &room,
                socket.within(room.clone()).sockets().unwrap().len(),
            );
        }
    }

    socket.on(
        "create",
        |socket: SocketRef, pool: State<PgPool>| async move {
            if !socket.rooms().unwrap().is_empty() {
                socket
                    .emit("created-room", socket.rooms().unwrap().first())
                    .unwrap();
                println!("{} Already in a room", socket.id);
                return;
            }

            let room = match add_room(socket.id, &pool).await {
                Err(e) => {
                    tracing::error!("{:?}", e);
                    return;
                }
                Ok(c) => c,
            };

            tracing::info!("Creating room: {:?}", room);
            socket.leave_all().unwrap();
            socket.join(room.clone()).unwrap();
            emit_update_room(
                &socket,
                &room,
                socket.within(room.clone()).sockets().unwrap().len(),
            );
        },
    );

    socket.on(
        "join",
        |socket: SocketRef, Data::<String>(room), pool: State<PgPool>| async move {
            if room.len() != ROOM_CODE_LENGTH {
                return;
            }
            tracing::info!("Joining room: {:?}", room);
            let room_error = join_room(socket.id, room.clone(), &pool).await;
            if let Err(e) = &room_error {
                if let Error::RoomFull(Some(player)) = &e {
                    tracing::warn!("{:?}", e);
                    update_sid(player, socket.id.as_str(), &pool).await.unwrap();
                    let data = get_game_state(socket.id.as_str(), &room, &pool).await.unwrap();
                    socket
                        .emit(
                            "restore",
                            serde_json::json!({"turn": data.0, "player": data.1, "opponent": data.2}),
                        )
                        .unwrap();
                } else {
                    tracing::error!("{:?}", e);
                    return;
                }
            }
            socket.leave_all().unwrap();
            socket.join(room.clone()).unwrap();

            let users = socket.within(room.clone()).sockets().unwrap().len();
            emit_update_room(&socket, &room, users);

            if users != 2 || room_error.is_err() {
                return;
            }
            let ack_stream = socket
                .within(room.clone())
                .emit_with_ack::<Vec<Board>>("upload", ())
                .unwrap();
            ack_stream
                .for_each(|(id, ack)| {
                    let pool = pool.clone();
                    async move {
                        match ack {
                            Ok(mut ack) => {
                                if let Err(e) = add_board(id, ack.data.pop().unwrap(), &pool).await
                                {
                                    tracing::error!("{:?}", e);
                                }
                            }
                            Err(err) => tracing::error!("Ack error, {}", err),
                        }
                    }
                })
                .await;
            if let Err(e) = start(socket.id, room.clone(), &pool).await {
                tracing::error!("{:?}", e);
                return;
            }
            tracing::info!("Game started");
            socket
                .within(room.clone())
                .emit("turnover", socket.id)
                .unwrap();
        },
    );

    socket.on(
        "attack",
        |socket: SocketRef, Data::<[usize; 2]>([i, j]), pool: State<PgPool>| async move {
            let (hit, sunk) = match attack(socket.id, (i, j), &pool).await {
                Ok(res) => res,
                Err(e) => {
                    tracing::error!("{:?}", e);
                    return;
                }
            };
            tracing::info!("Attacking at: ({}, {}), result: {:?}", i, j, hit);
            socket
                .within(socket.rooms().unwrap().first().unwrap().clone())
                .emit(
                    "attacked",
                    serde_json::json!({"by": socket.id.as_str(), "at": [i, j], "hit": hit, "sunk": sunk}),
                )
                .unwrap();
        },
    );

    socket.on(
        "leave",
        |socket: SocketRef, pool: State<PgPool>| async move {
            tracing::info!("Leaving Rooms: {:?}", socket.id);
            leave_and_inform(&socket, &pool, true).await;
        },
    );

    socket.on_disconnect(|socket: SocketRef, pool: State<PgPool>| async move {
        tracing::info!("Disconnecting: {:?}", socket.id);
        leave_and_inform(&socket, &pool, false).await;
    });
}

async fn leave_and_inform(socket: &SocketRef, pool: &PgPool, delete: bool) {
    let room = socket
        .rooms()
        .unwrap()
        .first()
        .map(|s| s.to_string())
        .or(get_room(socket.id, pool).await.unwrap());
    let Some(room) = room else {
        return;
    };
    let ops = socket.within(room.clone());
    socket.leave_all().unwrap();
    emit_update_room(socket, &room.to_string(), ops.sockets().unwrap().len());
    let sid = socket.id.as_str();
    if let Err(e) = if delete {
        delete_sid(sid, pool).await
    } else {
        to_delete_sid(sid, pool).await
    } {
        tracing::error!("{:?}", e);
    }
}

fn emit_update_room(socket: &SocketRef, room: &String, users: usize) {
    socket
        .within(room.clone())
        .emit(
            "update-room",
            serde_json::json!({"room": &room, "users": users}),
        )
        .unwrap();
}
