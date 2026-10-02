#![forbid(unsafe_code)]
use nuvex_cli::{execute, CliError};

fn main() {
    let code = match execute(std::env::args_os()) {
        Ok(()) => 0,
        Err(error @ CliError::NotImplemented { .. }) => {
            eprintln!("{error}");
            2
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    std::process::exit(code);
}
