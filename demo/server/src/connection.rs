use axum::extract::ws::{Message, WebSocket};
use core::hash::Hash;
use futures::stream::{SplitSink, SplitStream};
use std::sync::Arc;
use tokio::sync::Mutex;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub enum SocketError {
    AxumError(axum::Error),
    ClientClose,
}

impl From<axum::Error> for SocketError {
    fn from(error: axum::Error) -> Self {
        SocketError::AxumError(error)
    }
}

#[derive(Clone)]
pub struct Connection {
    id: u64,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Connection {}

impl Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl Connection {
    pub fn new(sender: SplitSink<WebSocket, Message>) -> Connection {
        Connection {
            id: get_unique_id(),
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    pub async fn send(&self, message: impl Into<String>) -> Result<(), SocketError> {
        let message = message.into();

        use futures::SinkExt;
        let _ = self.sender.lock().await.send(Message::Text(message)).await?;
        Ok(())
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

pub struct ConnectionStream {
    sender: Connection,
    receiver: SplitStream<WebSocket>,
}

impl ConnectionStream {
    pub fn new(stream: WebSocket) -> (Connection, ConnectionStream) {
        use futures::stream::StreamExt;

        let (sender, receiver) = stream.split();
        let sender = Connection::new(sender);

        (sender.clone(), ConnectionStream { sender, receiver })
    }

    pub async fn expect_get_text_message(&mut self) -> Result<String, SocketError> {
        use futures::stream::StreamExt;

        let message = self.receiver.next().await.ok_or(SocketError::ClientClose)??;

        if let Message::Text(message) = message {
            Ok(message)
        } else {
            self.sender.send("Error user: Text message was expected").await?;
            Err(SocketError::ClientClose)
        }
    }
}
