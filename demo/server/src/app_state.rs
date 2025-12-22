use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::connection::{Connection, SocketError};

use std::sync::OnceLock;

#[derive(Clone)]
pub struct AppState {
    connections: Arc<RwLock<HashSet<Connection>>>,
    messages: Arc<RwLock<Vec<String>>>,
}

impl AppState {
    pub fn global() -> AppState {
        static STATE: OnceLock<AppState> = OnceLock::new();
        STATE.get_or_init(AppState::new).clone()
    }

    fn new() -> AppState {
        AppState {
            connections: Arc::new(RwLock::new(HashSet::new())),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_connection(&self, socket: &Connection) {
        let mut connections = self.connections.write().await;
        connections.insert(socket.clone());
    }

    pub async fn remove_connection(&self, user_name: &Connection) {
        let _ = self.connections.write().await.remove(user_name);
    }

    async fn add_message(&self, line: &str) {
        self.messages.write().await.push(line.to_string());
    }

    pub async fn message_from(&self, message: String) -> Result<(), SocketError> {
        self.add_message(&message).await;

        for connection in self.connections.read().await.iter() {
            let _ = connection.send(&message).await;
        }

        Ok(())
    }

    pub async fn send_all_prev_messages(&self, socket: &Connection) -> Result<(), SocketError> {
        let all_messages = self.messages.read().await;

        for message in all_messages.iter() {
            socket.send(message).await?;
        }

        Ok(())
    }
}
