use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexModel {
    pub run_js: String,
    pub wasm: String,
}

#[derive(Args, Debug, Clone)]
pub struct CommonOpts {
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,
}
