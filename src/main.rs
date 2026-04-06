use clap::{Parser, Subcommand};
use folio::commands::check;
use folio::infrastructure::{Filesystem, Output};

#[derive(Parser)]
#[command(name = "folio", about = "Tag-based plain-text accounting")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a ledger file
    Check { path: String },
}

fn main() {
    let cli = Cli::parse();
    let fs = Filesystem::create();
    let output = Output::create();

    let code = match cli.command {
        Commands::Check { path } => check::run(&path, &fs, &output),
    };

    std::process::exit(code);
}
