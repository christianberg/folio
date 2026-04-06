pub use parser::parse;
pub use types::{Ledger, ParseError, Posting, Tag, Transaction};

pub mod commands;
pub mod infrastructure;

mod parser;
mod serialiser;
mod types;

pub fn run(args: infrastructure::Args, fs: &infrastructure::Filesystem, output: &infrastructure::Output) -> i32 {
    match args.command {
        infrastructure::Command::Check { path } => commands::check::run(&path, fs, output),
        infrastructure::Command::Add { path } => {
            let today = chrono::Local::now().date_naive();
            let prompt = infrastructure::Prompt::create();
            commands::add::run(&path, today, fs, &prompt, output)
        }
    }
}
