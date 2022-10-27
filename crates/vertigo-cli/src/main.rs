mod build;
mod new;
mod logs;

use clap::{Parser, Subcommand};

use build::BuildOpts;
use new::NewOpts;

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
}

fn main() -> Result<(), i32> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build(opts) => {
            build::run(opts)
        }
        Command::New(opts) => {
            new::run(opts)
        }
    }
}
