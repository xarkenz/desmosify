use std::process::ExitCode;
use clap::Parser;
use desmosify::cli::{invoke_wrapper, DesmosifyArgs};

fn main() -> ExitCode {
    invoke_wrapper(&DesmosifyArgs::parse())
}
