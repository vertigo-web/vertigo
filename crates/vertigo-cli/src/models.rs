use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexModel {
    pub run_js: String,
    pub wasm: String
}
