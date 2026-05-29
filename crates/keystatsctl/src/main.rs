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
    /// Show history statistics with terminal charts
    History {
        /// Number of days to display
        #[arg(long, default_value_t = 7)]
        days: u32,
        /// Show key presses only
        #[arg(long)]
        keys: bool,
        /// Show mouse clicks only
        #[arg(long)]
        clicks: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => commands::status(),
        Commands::Doctor => commands::doctor(),
        Commands::History { days, keys, clicks } => commands::history(days, keys, clicks),
    }
}
