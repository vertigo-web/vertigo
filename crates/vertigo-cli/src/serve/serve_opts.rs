use clap::Args;

use crate::commons::models::CommonOpts;
use crate::commons::parse_key_val;

#[derive(Args, Clone, Debug)]
pub struct ServeOpts {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[clap(flatten)]
    pub inner: ServeOptsInner,
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

    /// Setting the parameters `--env api=http://domain.com/api --env api2=http://domain.com/api2`
    #[arg(long, value_parser = parse_key_val::<String, String>)]
    pub env: Vec<(String, String)>,
}
