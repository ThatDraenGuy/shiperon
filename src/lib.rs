pub mod analyzer;
pub mod ast;

pub mod codegen;
pub mod config;

use std::error::Error;
use std::path::Path;

pub use config::CompilerConfig;
pub use config::ShipFeature;

pub mod diagnostics;

pub mod lexer;
pub use lexer::Lexer;
pub use lexer::TokenRegistry;

pub mod parser;
pub use parser::Parser;

pub mod source;
pub use source::ByteSource;

pub mod stdlib;
pub use stdlib::{ShipStdLib, StdlibCtx};

use crate::analyzer::Analyzer;
use crate::codegen::compile;
use crate::diagnostics::Renderable;
use crate::stdlib::stdlib;

pub fn process(input: &str, output: &Path, config: CompilerConfig) -> Result<(), Box<dyn Error>> {
    let debug = config.debug;
    let parser = Parser::new(Lexer::of_str(input), config);
    let parse_data = parser.consume_parse();

    let mut all_diagnostics = parse_data.diagnostics;
    let lib = stdlib();
    if let Some(ast) = parse_data.program {
        let (ctx, mut diagnostics) = Analyzer::new(ast).analyze(lib);
        all_diagnostics.append(&mut diagnostics);

        if all_diagnostics.is_empty() {
            compile(ctx, output, debug)?;
        }
    }
    for item in &all_diagnostics {
        println!("{}\n", item.render(&parse_data.src));
    }
    Ok(())
}
