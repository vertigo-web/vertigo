use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct IndexModel {
    pub run_js: String,
    pub wasm: String
}
