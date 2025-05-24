use clap::{Parser, Subcommand};
use env_logger::Builder;
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
    Builder::new()
        .parse_env("RUST_LOG")
        .filter(Some("cranelift_codegen"), log::LevelFilter::Warn)
        .filter(Some("wasmtime_cranelift::compiler"), log::LevelFilter::Warn)
        .init();

    let cli = Cli::parse();

    let ret = match cli.command {
        Command::Build(opts) => build::run(opts),
        Command::New(opts) => new::run(opts),
        Command::Serve(opts) => serve::run(opts, None).await,
        Command::Watch(opts) => watch::run(opts).await,
    };

    // Tokio doesn't proparate error codes to shell, so do it manually
    if let Err(err) = ret {
        exit(err as i32);
    }

    Ok(())
}
