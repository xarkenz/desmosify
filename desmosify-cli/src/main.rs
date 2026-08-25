use std::process::ExitCode;
use clap::Parser;
use desmosify_cli::CommandLineArguments;

fn main() -> ExitCode {
    desmosify_cli::invoke(&CommandLineArguments::parse())
}
