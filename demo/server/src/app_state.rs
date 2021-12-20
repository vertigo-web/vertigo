use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::connection::{Connection, SocketError};

#[derive(Clone)]
pub struct AppState {
    connections: Arc<Mutex<HashSet<Connection>>>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            connections: Arc::new(Mutex::new(HashSet::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_connection(&self, socket: &Connection) {
        let mut connections = self.connections.lock().await;
        connections.insert(socket.clone());
    }

    pub async fn remove_connection(&self, user_name: &Connection) {
        let _ = self.connections.lock().await.remove(user_name);
    }

    async fn add_message(&self, line: &str) {
        self.messages.lock().await.push(line.to_string());
    }

    pub async fn message_from(&self, message: String) -> Result<(), SocketError> {
        self.add_message(&message).await;

        for connection in self.connections.lock().await.iter() {
            let _ = connection.send(&message).await;
        }

        Ok(())
    }

    pub async fn send_all_prev_messages(&self, socket: &Connection) -> Result<(), SocketError> {
        let all_messages = self.messages.lock().await;

        for message in all_messages.iter() {
            socket.send(message).await?;
        }

        Ok(())
    }
}
