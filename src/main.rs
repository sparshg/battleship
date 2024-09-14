mod game;

use axum::{
    routing::{get, post},
    Json, Router,
};
use game::Board;
use serde::Serialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(game::create_board_route));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
