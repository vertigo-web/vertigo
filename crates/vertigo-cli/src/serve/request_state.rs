use std::collections::HashMap;

#[derive(Clone)]
pub struct RequestState {
    pub url: String,
    pub env: HashMap<String, String>,
}

impl RequestState {
    pub fn env(&self, name: impl Into<String>) -> Option<String> {
        let name = name.into();
        let Some(value) = self.env.get(&name) else {
            return None;
        };

        Some(value.clone())
    }
}

