#![deny(rust_2018_idioms)]

use std::net::SocketAddr;

use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

mod app_state;
mod connection;

use app_state::AppState;
use connection::{Connection, ConnectionStream, SocketError};

async fn index() -> Html<String> {
    Html("demo - api index".to_string())
}

#[tokio::main]
async fn main() {
    println!("Server start on 127.0.0.1:3333 ...");

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));

    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        Ok(()) => {
            println!("server stop - ok");
        }
        Err(error) => {
            println!("error run: {error:?}");
        }
    }
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(websocket)
}

async fn websocket(stream: WebSocket) {
    // By splitting we can send and receive at the same time.

    let (sender, receiver) = ConnectionStream::new(stream);

    let result = websocket_process(&sender, receiver).await;

    match result {
        Ok(()) => {}
        Err(err) => match err {
            SocketError::AxumError(err) => {
                println!("Client disconnected -> {err}");
            }

            SocketError::ClientClose => {
                println!("Client disconnected");
            }
        },
    }
}

async fn websocket_process(
    sender: &Connection,
    mut receiver: ConnectionStream,
) -> Result<(), SocketError> {
    sender
        .send(format!("New connection, id={}", sender.get_id()))
        .await?;

    println!("New connection: {}", sender.get_id());

    let state = AppState::global();
    state.add_connection(sender).await;
    state.send_all_prev_messages(sender).await?;

    let result = websocket_loop(&mut receiver).await;

    println!("Connection close: {}", sender.get_id());

    state.remove_connection(sender).await;

    result
}

async fn websocket_loop(receiver: &mut ConnectionStream) -> Result<(), SocketError> {
    let state = AppState::global();
    loop {
        let message = receiver.expect_get_text_message().await?;
        state.message_from(message).await?;
    }
}
