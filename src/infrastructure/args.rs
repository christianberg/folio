use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "folio", about = "Tag-based plain-text accounting")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Validate a ledger file
    Check { path: String },
    /// Interactively add a transaction
    Add { path: String },
}

impl Args {
    pub fn create() -> Self {
        Self::parse()
    }

    pub fn create_null(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::parse_from(args.into_iter().map(|a| a.into()))
    }
}
