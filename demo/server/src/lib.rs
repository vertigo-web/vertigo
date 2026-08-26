#![deny(rust_2018_idioms)]

//! The demo's API side: the chat and collection websockets, the lazy-list CRUD store, and
//! stand-ins for the two public APIs the demo otherwise calls out to.
//!
//! Exposed as a library so that the browser test in `tests/demo` can start the very same
//! routes on a port of its own, instead of reimplementing them. The binary in `main.rs` is a
//! thin wrapper that picks the well-known port.

use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, dev::Server, rt, web};

mod app_state;
mod client_message;
mod connection;
mod items;
mod stub_api;
mod ws_collection;

/// Re-exported so a caller can hold on to what [`start_background`] returns without taking a
/// dependency on actix itself.
pub use actix_web::dev::ServerHandle;

use app_state::AppState;
use client_message::ClientMessage;
use connection::{Connection, ConnectionStream, SocketError};

/// Bind the API server, ready to be awaited (or driven from another thread).
///
/// Returns before serving starts, so a caller that needs to stop it later can keep the
/// [`Server::handle`] it hands out.
pub fn build_server(host: &str, port: u16) -> std::io::Result<Server> {
    let items_state = items::new_state();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(items_state.clone())
            .route("/", web::get().to(|| async { "demo - api index" }))
            .route("/ws", web::get().to(websocket_handler))
            .route("/ws-collection", web::get().to(ws_collection::handler))
            // Lazy list demo
            .route("/api/items", web::get().to(items::list))
            .route("/api/items", web::post().to(items::create))
            .route("/api/items/{id}", web::get().to(items::get_one))
            .route("/api/items/{id}", web::put().to(items::update))
            .route("/api/items/{id}", web::delete().to(items::delete))
            .configure(stub_api::configure)
    })
    .bind((host, port))?
    .run();

    Ok(server)
}

/// Start the API server on a thread of its own and return the handle that stops it.
///
/// Actix wants its own runtime, which is why this is a thread with a `System` rather than a
/// task. Keeping it here rather than in the caller means the browser test does not have to
/// depend on actix at all: it gets a handle and calls `stop` on it.
///
/// Blocks until the port is bound, so a caller can connect straight after it returns.
pub fn start_background(host: &str, port: u16) -> std::io::Result<ServerHandle> {
    let address = format!("{host}:{port}");
    let host = host.to_string();
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let system = rt::System::new();

        let server = match system.block_on(async { build_server(&host, port) }) {
            Ok(server) => {
                // Sent before serving starts; `run()` has already bound the port.
                let _ = sender.send(Ok(server.handle()));
                server
            }
            Err(err) => {
                let _ = sender.send(Err(err));
                return;
            }
        };

        if let Err(err) = system.block_on(server) {
            println!("demo api server stopped with an error: {err}");
        }
    });

    match receiver.recv() {
        Ok(result) => result,
        Err(err) => Err(std::io::Error::other(format!(
            "demo api server thread died before binding {address}: {err}"
        ))),
    }
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
