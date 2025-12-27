use clap::{Parser, Subcommand};
use std::process::exit;

pub mod build;
mod commons;
pub mod new;
pub mod serve;
pub mod watch;

pub use build::BuildOpts;
pub use commons::models::CommonOpts;
pub use new::NewOpts;
pub use serve::ServeOpts;
pub use watch::WatchOpts;

// For bundling with custom back-end
pub use serve::vertigo_install;

use commons::logging::setup_logging;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New(NewOpts),
    Build(BuildOpts),
    Serve(ServeOpts),
    Watch(WatchOpts),
}

#[tokio::main]
pub async fn main() -> Result<(), i32> {
    let cli = Cli::parse();

    setup_logging(&cli.command);

    let ret = match cli.command {
        Command::Build(opts) => build::run(opts),
        Command::New(opts) => new::run(opts),
        Command::Serve(opts) => serve::run(opts, None).await,
        Command::Watch(opts) => watch::run(opts).await,
    };

    // Tokio doesn't propagate error codes to shell, so do it manually
    if let Err(err) = ret {
        exit(err as i32);
    }

    Ok(())
}
