use std::path::PathBuf;
use std::process::ExitCode;
use clap::Parser as ClapParser;
use crate::ast::parse::Parser;
use crate::sema::context::GlobalContext;
use crate::sema::interpret::interpret_program;
use crate::token::scan::Scanner;

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
pub struct DesmosifyArgs {
    #[doc = "The paths of source code files to compile into a single program"]
    #[arg(value_name = "source_paths")]
    pub source_paths: Vec<PathBuf>,
    #[doc = "The path where compilation output will be written to"]
    #[arg(short = 'o', long = "out", value_name = "output_path")]
    pub output_path: PathBuf,
    #[doc = "The name of the compilation target"]
    #[arg(short = 't', long = "target", value_name = "target_name")]
    pub target_name: String,
}

pub fn invoke(args: &DesmosifyArgs) -> crate::Result<()> {
    let mut target = crate::target::new_target_by_name(&args.target_name)?;

    let mut declarations = Vec::new();

    for (source_id, source_path) in args.source_paths.iter().enumerate() {
        println!("Parsing '{}'...", source_path.display());

        let mut scanner = Scanner::from_path(source_id, source_path)?;
        let mut parser = Parser::new(&mut scanner)?;

        while let Some(declaration) = parser.parse_declaration()? {
            declarations.push(declaration);
        }
    }

    println!("Analyzing program...");

    let context = GlobalContext::from_declarations(declarations, target.as_ref())?;
    let program = interpret_program(&args.source_paths, target.as_mut(), &context)?;

    println!("Compiling program...");

    target.compile_to(&program, &args.output_path)?;

    println!("Successfully written to '{}'.", args.output_path.display());

    Ok(())
}

pub fn invoke_wrapper(args: &DesmosifyArgs) -> ExitCode {
    match invoke(args) {
        Ok(()) => {
            println!("\x1b[32mFinished\x1b[0m");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("\x1b[31m{}\x1b[0m", error.to_string_with_context(&args.source_paths));
            ExitCode::FAILURE
        }
    }
}
