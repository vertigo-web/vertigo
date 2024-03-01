use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexModel {
    pub run_js: String,
    pub wasm: String,
}
