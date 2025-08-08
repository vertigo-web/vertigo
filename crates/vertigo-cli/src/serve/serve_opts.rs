use clap::Args;

use crate::commons::models::CommonOpts;
use crate::commons::parse_key_val;

#[derive(Args, Clone, Debug)]
pub struct ServeOpts {
    #[clap(flatten)]
    pub inner: ServeOptsInner,
    #[clap(flatten)]
    pub common: CommonOpts,
}

#[derive(Args, Clone, Debug)]
pub struct ServeOptsInner {
    #[arg(long, default_value_t = {"127.0.0.1".into()})]
    pub host: String,
    #[arg(long, default_value_t = {4444})]
    pub port: u16,

    /// sets up proxy: `--proxy /path=http://domain.com/path`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub proxy: Vec<(String, String)>,

    /// Define a mount point for the app in public url.
    ///
    /// (For example an nginx's location in which proxy_pass to the app was defined)
    #[arg(long, default_value_t = {"/".to_string()})]
    pub mount_point: String,

    /// Setting the parameters `--env api=http://domain.com/api --env api2=http://domain.com/api2`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub env: Vec<(String, String)>,

    /// Whether to add <link rel="preload"> tag to <head> to make the browser load WASM earlier
    #[arg(long, default_value_t = {false})]
    pub wasm_preload: bool,
}
