use actix_ws::{Message, MessageStream, Session};
use core::hash::Hash;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, thiserror::Error)]
pub enum SocketError {
    #[error("Actix error: {0}")]
    ActixError(#[from] actix_ws::ProtocolError),
    #[error("Client close")]
    ClientClose,
    #[error("Send error: {0}")]
    SendError(#[from] actix_ws::Closed),
}

#[derive(Clone)]
pub struct Connection {
    id: u64,
    sender: Session,
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
    pub fn new(sender: Session) -> Connection {
        Connection {
            id: get_unique_id(),
            sender,
        }
    }

    pub async fn send(&self, message: impl Into<String>) -> Result<(), SocketError> {
        let message = message.into();
        let mut session = self.sender.clone();
        session.text(message).await.map_err(SocketError::from)
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

pub struct ConnectionStream {
    sender: Connection,
    receiver: MessageStream,
}

impl ConnectionStream {
    pub fn new(session: Session, receiver: MessageStream) -> (Connection, Self) {
        let sender = Connection::new(session);

        (sender.clone(), Self { sender, receiver })
    }

    pub async fn expect_get_text_message(&mut self) -> Result<String, SocketError> {
        use futures::stream::StreamExt;

        let message = self
            .receiver
            .next()
            .await
            .ok_or(SocketError::ClientClose)??;

        match message {
            Message::Text(text) => Ok(text.to_string()),
            Message::Close(_) => Err(SocketError::ClientClose),
            _ => {
                self.sender
                    .send("Error user: Text message was expected")
                    .await?;
                Err(SocketError::ClientClose)
            }
        }
    }
}
