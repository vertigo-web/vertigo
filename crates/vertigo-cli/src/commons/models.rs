use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexModel {
    pub run_js: String,
    pub wasm: String,
}

#[derive(Args, Debug, Clone)]
pub struct CommonOpts {
    /// Directory with wasm_run.js and compiled .wasm file. Should be the same during build and serve.
    #[arg(long, default_value_t = {"./build".to_string()})]
    pub dest_dir: String,

    /// Whether to use local time in logging.
    ///
    /// This defaults to true for `watch` command and to false for all other commands.
    #[arg(long, hide_short_help(true))]
    pub log_local_time: Option<bool>,
}
