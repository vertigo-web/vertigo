use clap::Args;

use crate::build::{BuildOpts, BuildOptsInner};
use crate::commons::models::CommonOpts;
use crate::serve::ServeOptsInner;
use crate::ServeOpts;

#[derive(Args, Debug, Clone)]
pub struct WatchOpts {

    #[arg(long, default_value_t = {5555})]
    pub port_watch: u16,

    /// Add more directories to be watched for code changes
    #[arg(long)]
    pub add_watch_path: Vec<String>,

    /// What ignore lists to search for in watched directories (space-separated, .gitignore format)
    #[arg(long, default_value_t = {".gitignore".to_string()}, hide_short_help(true))]
    pub watch_ignore_lists: String,

    /// Additional globs to ignore in every watch path (space-separated, .gitignore format)
    #[arg(long, default_value_t = {"**/*.swp **/*.swx **/*.rs.bk".to_string()}, hide_short_help(true))]
    pub global_ignores: String,

    #[clap(flatten)]
    pub serve: ServeOptsInner,

    #[clap(flatten)]
    pub build: BuildOptsInner,

    #[clap(flatten)]
    pub common: CommonOpts,
}

impl WatchOpts {
    pub fn to_build_opts(&self) -> BuildOpts {
        BuildOpts {
            common: self.common.clone(),
            inner: self.build.clone(),
        }
    }

    pub fn to_serve_opts(&self) -> (ServeOpts, u16) {
        (
            ServeOpts {
                common: self.common.clone(),
                inner: self.serve.clone(),
            },
            self.port_watch,
        )
    }
}
