pub mod ast;
pub mod cli;
pub mod desmos;
pub mod error;
pub mod sema;
pub mod source;
pub mod target;
pub mod token;

pub use error::*;
pub use source::*;

#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Debug)]
pub struct CompileOptions {
    /// The name of the compilation target.
    #[cfg_attr(feature = "clap", arg(short = 't', long = "target", value_name = "target_name"))]
    pub target_name: String,
    /// How to handle emitting fragile functions for Desmos.
    #[cfg_attr(feature = "clap", arg(long = "fragile-strategy"))]
    pub fragile_strategy: desmos::builder::fragile::FragileStrategy,
}

pub fn compile(options: &CompileOptions) -> crate::Result<()> {
    let mut target = target::create_target(options)?;

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
