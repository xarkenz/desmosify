pub mod ast;
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

pub fn compile(sources: &SourceFiles, options: &CompileOptions) -> crate::Result<String> {
    let mut target = target::create_target(options)?;

    let mut declarations = Vec::new();

    for source_handle in sources.handles() {
        println!("Parsing '{}'...", source_handle.file(sources).path.display());

        let mut scanner = token::scan::Scanner::new(sources, source_handle);
        let mut parser = ast::parse::Parser::new(&mut scanner)?;

        while let Some(declaration) = parser.parse_declaration()? {
            declarations.push(declaration);
        }
    }

    println!("Analyzing program...");

    let context = sema::context::GlobalContext::from_declarations(declarations, target.as_ref())?;
    let program = sema::interpret::interpret_program(sources, target.as_mut(), &context)?;

    println!("Compiling program...");

    target.generate_output(&program)
}
