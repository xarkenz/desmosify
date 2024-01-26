use crate::target::Target;

use std::io::{Read, Write};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct DesmosifyArgs {
    #[arg(short, long)]
    src: Vec<String>,
    #[arg(short, long)]
    out: String,
    #[arg(long)]
    debug: bool,
}

impl DesmosifyArgs {
    pub fn source_paths(&self) -> &[String] {
        &self.src
    }

    pub fn output_path(&self) -> &str {
        &self.out
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}

pub fn parse_command_line_args() -> DesmosifyArgs {
    DesmosifyArgs::parse()
}

pub fn invoke(args: &DesmosifyArgs) -> Result<(), crate::DesmosifyError> {
    for source_path in args.source_paths() {
        println!("Compiling '{source_path}'...");

        let mut source_file = std::fs::File::open(source_path)
            .map_err(|err| crate::DesmosifyError::new(err.to_string(), None, None))?;
        let mut source = String::new();
        source_file.read_to_string(&mut source)
            .map_err(|err| crate::DesmosifyError::new(err.to_string(), None, None))?;

        let tokens = crate::token::tokenize(&source)?;
        let (signatures, mut definitions) = crate::syntax::parse(&tokens)?;
        crate::semantics::analyze(&signatures, &mut definitions)?;

        let target = crate::target::desmos::GeometryTarget;
        let output = target.compile(&definitions, &signatures);

        let output_path = args.output_path();
        let mut output_file = std::fs::File::create(output_path)
            .map_err(|err| crate::DesmosifyError::new(err.to_string(), None, None))?;
        write!(output_file, "{output}")
            .map_err(|err| crate::DesmosifyError::new(err.to_string(), None, None))?;

        println!("Successfully written to '{output_path}'.");
    }

    Ok(())
}