mod board;
mod game;
use axum::Router;
use board::Board;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use game::{add_board, add_room, attack, disconnect, join_room, start, ROOM_CODE_LENGTH};
use rand::Rng;
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

fn on_connect(socket: SocketRef) {
    tracing::info!("Connected: {:?}", socket.id);
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

            let room: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(ROOM_CODE_LENGTH)
                .map(|x| char::to_ascii_uppercase(&(x as char)))
                .collect();
            tracing::info!("Creating room: {:?}", room);

            if let Err(e) = add_room(socket.id, room.clone(), &pool).await {
                tracing::error!("{:?}", e);
                return;
            }
            socket.leave_all().unwrap();
            socket.join(room.clone()).unwrap();
            socket.emit("joined-room", &room).unwrap();
        },
    );

    socket.on(
        "join",
        |socket: SocketRef, Data::<String>(room), pool: State<PgPool>| async move {
            if room.len() != ROOM_CODE_LENGTH {
                return;
            }
            tracing::info!("Joining room: {:?}", room);
            if let Err(e) = join_room(socket.id, room.clone(), &pool).await {
                tracing::error!("{:?}", e);
                return;
            }
            socket.leave_all().unwrap();
            socket.join(room.clone()).unwrap();
            socket.emit("joined-room", &room).unwrap();

            if socket.within(room.clone()).sockets().unwrap().len() != 2 {
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

    socket.on_disconnect(|socket: SocketRef, pool: State<PgPool>| async move {
        tracing::info!("Disconnecting: {:?}", socket.id);
        socket.leave_all().unwrap();
        if let Err(e) = disconnect(socket.id, &pool).await {
            tracing::error!("{:?}", e);
        }
    });
}
