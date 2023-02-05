mod build;
mod new;
mod serve;
mod models;
mod check_env;
mod command;
mod run_parallel;

use clap::{Parser, Subcommand};

use build::BuildOpts;
use new::NewOpts;
use run_parallel::RunParallelOpts;
use serve::ServeOpts;

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
    RunParallel(RunParallelOpts),
}

#[tokio::main]
pub async fn main() -> Result<(), i32> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("cargo::core::compiler"), log::LevelFilter::Off)
        .init();

    let _ = check_env::check_env().await?;

    let cli = Cli::parse();
    match cli.command {
        Command::Build(opts) => {
            build::run(opts)
        }
        Command::New(opts) => {
            new::run(opts)
        }
        Command::Serve(opts) => {
            serve::run(opts).await
        }
        Command::RunParallel(opts) => {
            run_parallel::run_parallel(opts).await
        }
    }
}
