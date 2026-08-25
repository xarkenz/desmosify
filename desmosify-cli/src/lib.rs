use std::path::PathBuf;
use std::process::ExitCode;
use desmosify::{SourceFile, SourceFiles};

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
pub struct CommandLineArguments {
    /// The paths of source code files to compile into a single program.
    #[arg(value_name = "source_paths")]
    pub source_paths: Vec<PathBuf>,
    /// The path where compilation output will be written to.
    #[arg(short = 'o', long = "out", value_name = "output_path")]
    pub output_path: PathBuf,
    #[command(flatten)]
    pub compile_options: desmosify::CompileOptions,
}

pub fn invoke(arguments: &CommandLineArguments) -> ExitCode {
    let mut read_errors = Vec::new();
    let source_contents: Vec<String> = arguments.source_paths
        .iter()
        .map(|path| match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => {
                read_errors.push((path.clone(), error));
                Default::default()
            }
        })
        .collect();

    if !read_errors.is_empty() {
        for (path, error) in read_errors {
            println!("\x1b[31mError: could not read file at '{}': {error}\x1b[0m", path.display());
        }
        return ExitCode::FAILURE
    }

    let sources: SourceFiles = std::iter::zip(&arguments.source_paths, &source_contents)
        .map(|(path, content)| SourceFile {
            path,
            content,
        })
        .collect();

    let output = match desmosify::compile(&sources, &arguments.compile_options) {
        Ok(output) => output,
        Err(error) => {
            println!("\x1b[31mError: {}\x1b[0m", error.display_with_context(&sources));
            return ExitCode::FAILURE
        }
    };

    if let Err(error) = arguments.output_path
        .parent()
        .map_or(Ok(()), |output_dir| std::fs::create_dir_all(output_dir))
        .and_then(|()| std::fs::File::create(&arguments.output_path))
        .and_then(|mut file| {
            use std::io::Write;
            write!(file, "{output}")
        })
    {
        println!("\x1b[31mError: could not save output to '{}': {error}\x1b[0m", arguments.output_path.display());
        return ExitCode::FAILURE
    }

    println!("\x1b[32mFinished\x1b[0m");
    ExitCode::SUCCESS
}
