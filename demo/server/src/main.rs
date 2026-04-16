#![deny(rust_2018_idioms)]

use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, rt, web};

mod app_state;
mod client_message;
mod connection;
mod items;

use app_state::AppState;
use client_message::ClientMessage;
use connection::{Connection, ConnectionStream, SocketError};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server start on 127.0.0.1:3333 ...");

    let items_state = items::new_state();

    HttpServer::new(move || {
        App::new()
            .app_data(items_state.clone())
            .route("/", web::get().to(|| async { "demo - api index" }))
            .route("/ws", web::get().to(websocket_handler))
            // Lazy list demo
            .route("/api/items", web::get().to(items::list))
            .route("/api/items", web::post().to(items::create))
            .route("/api/items/{id}", web::get().to(items::get_one))
            .route("/api/items/{id}", web::put().to(items::update))
            .route("/api/items/{id}", web::delete().to(items::delete))
    })
    .bind(("127.0.0.1", 3333))?
    .run()
    .await
}

async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    rt::spawn(websocket(session, msg_stream));

    Ok(response)
}

async fn websocket(session: actix_ws::Session, msg_stream: actix_ws::MessageStream) {
    // By splitting we can send and receive at the same time.

    let (sender, receiver) = ConnectionStream::new(session, msg_stream);

    let result = websocket_process(&sender, receiver).await;

    if let Err(err) = result {
        println!("Client disconnected -> {err}");
    }
}

async fn websocket_process(
    sender: &Connection,
    mut receiver: ConnectionStream,
) -> Result<(), SocketError> {
    let id = sender.get_id();
    let welcome = ClientMessage::Info {
        message: format!("New connection, id={id}"),
    };
    sender.send(welcome.to_json()?).await?;

    println!("New connection: {id}");

    let state = AppState::global();
    state.add_connection(sender).await;
    state.send_all_prev_messages(sender).await?;

    let result = websocket_loop(&mut receiver).await;

    println!("Connection close: {id}");

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
