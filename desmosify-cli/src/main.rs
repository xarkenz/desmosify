use std::path::PathBuf;
use std::process::ExitCode;
use desmosify::CompileOptions;

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// The paths of source code files to compile into a single program.
    #[arg(value_name = "source_paths")]
    pub source_paths: Vec<PathBuf>,
    /// The path where compilation output will be written to.
    #[arg(short = 'o', long = "out", value_name = "output_path")]
    pub output_path: PathBuf,
    #[command(flatten)]
    pub compile_options: CompileOptions,
}

fn main() -> ExitCode {
    invoke_wrapper(&CliArgs::parse())
}
