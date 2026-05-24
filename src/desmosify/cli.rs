use std::io::Write;
use std::path::{Path, PathBuf};
use clap::Parser as ClapParser;
use crate::ast::parse::Parser;
use crate::desmos::ToJson;
use crate::desmos::target::DesmosGeometryTarget;
use crate::sema::context::GlobalContext;
use crate::sema::interpret::interpret_program;
use crate::target::Target;
use crate::token::scan::Scanner;

#[derive(ClapParser, Debug)]
#[command(author, version)]
pub struct DesmosifyArgs {
    #[arg(short, long)]
    pub src: Vec<PathBuf>,
    #[arg(short, long)]
    pub out: PathBuf,
    #[arg(long)]
    pub debug: bool,
}

impl DesmosifyArgs {
    pub fn source_paths(&self) -> &[PathBuf] {
        &self.src
    }

    pub fn output_path(&self) -> &Path {
        &self.out
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

pub fn parse_command_line_args() -> DesmosifyArgs {
    DesmosifyArgs::parse()
}

pub fn invoke(args: &DesmosifyArgs) -> crate::Result<()> {
    let mut declarations = Vec::new();

    for (source_id, source_path) in args.source_paths().iter().enumerate() {
        println!("Parsing '{}'...", source_path.display());

        let mut scanner = Scanner::from_path(source_id, source_path)?;
        let mut parser = Parser::new(&mut scanner)?;

        while let Some(declaration) = parser.parse_declaration()? {
            declarations.push(declaration);
        }
    }

    println!("Analyzing program...");

    let context = GlobalContext::from_declarations(declarations)?;
    let program = interpret_program(&context)?;

    println!("Compiling program...");

    let graph = DesmosGeometryTarget.compile(&program)?;

    let output_path = args.output_path();
    let mut output_file = std::fs::File::create(output_path)
        .map_err(|cause| crate::Error {
            kind: crate::ErrorKind::OutputFileOpen {
                path: output_path.into(),
                cause,
            },
            span: None,
        })?;

    writeln!(output_file, "{}", graph.to_json())
        .map_err(|cause| crate::Error {
            kind: crate::ErrorKind::OutputFileWrite {
                path: output_path.into(),
                cause,
            },
            span: None,
        })?;

    println!("Successfully written to '{}'.", output_path.display());

    Ok(())
}
