use crate::infrastructure::{Filesystem, Output};
use crate::parse;

pub fn run(path: &str, fs: &Filesystem, output: &Output) -> i32 {
    let content = match fs.read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            output.eprintln(&format!("error: {e}"));
            return 1;
        }
    };

    match parse(&content) {
        Ok(_) => {
            output.println(&format!("{path}: ok"));
            0
        }
        Err(e) => {
            output.eprintln(&format!("{path}: {e}"));
            1
        }
    }
}
