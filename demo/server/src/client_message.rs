#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientMessage {
    Info { message: String },
    UserMessage { message: String },
}

impl ClientMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
