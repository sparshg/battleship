mod game;
use std::{str::FromStr, sync::Arc};

use axum::Router;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use game::{join_room, Board, ROOM_CODE_LENGTH};
use rand::Rng;
use serde_json::Value;
use socketioxide::{
    adapter::Room,
    extract::{AckSender, Data, SocketRef, State},
    socket::Sid,
    SocketIo,
};
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
    join_room(
        Sid::from_str("aaaaaaaaaaaaaaaa").unwrap(),
        "AAAB".to_string(),
        &pool,
    )
    .await?;
    let (layer, io) = SocketIo::builder().with_state(pool).build_layer();
    // io.ns("/", on_connect);
    let app = Router::new()
        // .route("/", post(game::create_board_route))
        .layer(layer)
        .layer(CorsLayer::very_permissive());

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

// fn on_connect(socket: SocketRef, io: SocketIo) {
//     tracing::info!("Connected: {:?}", socket.id);
//     // tracing::info!(
//     //     "All rooms and sockets: {:?}",
//     //     io.rooms()
//     //         .unwrap()
//     //         .iter()
//     //         .map(|room| { (room, io.within(room.clone()).sockets().unwrap()) })
//     // );

//     socket.on(
//         "create",
//         |socket: SocketRef, store: State<Store>| async move {
//             if !socket.rooms().unwrap().is_empty() {
//                 socket
//                     .emit("created-room", socket.rooms().unwrap().first())
//                     .unwrap();
//                 println!("{} Already in a room", socket.id);
//                 return;
//             }

//             let room: String = rand::thread_rng()
//                 .sample_iter(&rand::distributions::Alphanumeric)
//                 .take(ROOM_CODE_LENGTH)
//                 .map(|x| char::to_ascii_uppercase(&(x as char)))
//                 .collect();
//             tracing::info!("Creating room: {:?}", room);
//             store.add_room(room.clone()).await;
//             store.join_room(socket.id, room.clone()).await.unwrap();
//             socket.leave_all().unwrap();
//             socket.join(room.clone()).unwrap();
//             socket.emit("created-room", &room).unwrap();
//         },
//     );

//     socket.on(
//         "join",
//         |socket: SocketRef, Data::<String>(room), store: State<Store>| async move {
//             if room.len() != ROOM_CODE_LENGTH {
//                 return;
//             }
//             tracing::info!("Joining room: {:?}", room);
//             store.join_room(socket.id, room.clone()).await.unwrap();
//             socket.leave_all().unwrap();
//             socket.join(room.clone()).unwrap();
//             if socket.within(room.clone()).sockets().unwrap().len() != 2 {
//                 return;
//             }
//             let ack_stream = socket
//                 .within(room.clone())
//                 .emit_with_ack::<Vec<Board>>("upload", ())
//                 .unwrap();
//             ack_stream
//                 .for_each(|(id, ack)| {
//                     let store = store.clone();
//                     async move {
//                         match ack {
//                             Ok(mut ack) => {
//                                 store.add_board(id, ack.data.pop().unwrap()).await.unwrap();
//                             }
//                             Err(err) => tracing::error!("Ack error, {}", err),
//                         }
//                     }
//                 })
//                 .await;
//             store.start(room.clone(), socket.id).await.unwrap();
//             tracing::info!("Game started");
//             socket.within(room.clone()).emit("turn", socket.id).unwrap();
//         },
//     );

//     socket.on(
//         "attack",
//         |socket: SocketRef, Data::<[usize; 2]>([i, j]), ack: AckSender, store: State<Store>| async move {
//             let res = store.attack(socket.id, (i, j)).await.unwrap();
//             tracing::info!("Attacking at: ({}, {}), result: {}", i, j, res);
//             ack.send(res).unwrap();
//         },
//     );

//     socket.on_disconnect(|socket: SocketRef, store: State<Store>| {
//         tracing::info!("Disconnecting: {:?}", socket.id);
//         socket.leave_all().unwrap();
//         // TODO: Delete room
//     });
// }
