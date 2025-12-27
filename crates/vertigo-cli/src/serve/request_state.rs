use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct RequestState {
    pub url: String,
    pub env: Arc<HashMap<String, String>>,
}

impl RequestState {
    pub fn env(&self, name: impl Into<String>) -> Option<String> {
        let name = name.into();
        let value = self.env.get(&name)?;

        Some(value.clone())
    }
}
