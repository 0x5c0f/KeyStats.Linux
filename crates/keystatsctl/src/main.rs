mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "keystatsctl")]
#[command(version, about = "KeyStats CLI – control and inspect the Linux daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show daemon status and today stats
    Status,
    /// Run permission diagnostic
    Doctor,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => commands::status(),
        Commands::Doctor => commands::doctor(),
    }
}
