#![deny(rust_2018_idioms)]
#![feature(map_try_insert)]

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Extension,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    routing::service_method_routing,
    AddExtensionLayer, Router,
};
use tower_http::services::ServeDir;

mod app_state;
mod connection;

use app_state::AppState;
use connection::{Connection, ConnectionStream, SocketError};

#[tokio::main]
async fn main() {
    println!("Server start on 127.0.0.1:3000 ...");

    let app_state = AppState::new();

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .fallback({
            use axum::error_handling::HandleErrorExt;
            service_method_routing::get(ServeDir::new("demo/build")).handle_error(|error: std::io::Error| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            })
        })
        .layer(AddExtensionLayer::new(app_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade, Extension(state): Extension<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: AppState) {
    // By splitting we can send and receive at the same time.

    let (sender, receiver) = ConnectionStream::new(stream);

    let result = websocket_process(&sender, receiver, state).await;

    match result {
        Ok(()) => {}
        Err(err) => match err {
            SocketError::AxumError(err) => {
                println!("Client disconnected -> {}", err);
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
    state: AppState,
) -> Result<(), SocketError> {
    sender.send(format!("New connection, id={}", sender.get_id())).await?;

    println!("New connection: {}", sender.get_id());

    state.add_connection(sender).await;
    state.send_all_prev_messages(sender).await?;

    let result = websocket_loop(&mut receiver, &state).await;

    println!("Connection close: {}", sender.get_id());

    state.remove_connection(sender).await;

    result
}

async fn websocket_loop(receiver: &mut ConnectionStream, state: &AppState) -> Result<(), SocketError> {
    loop {
        let message = receiver.expect_get_text_message().await?;
        state.message_from(message).await?;
    }
}
