pub mod analyzer;
pub mod ast;

pub mod config;

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
use crate::diagnostics::Renderable;
use crate::stdlib::stdlib;

pub fn process(input: &str, config: CompilerConfig) {
    let parser = Parser::new(Lexer::of_str(input), config);
    let parse_data = parser.consume_parse();

    let mut all_diagnostics = parse_data.diagnostics;
    let lib = stdlib();
    if let Some(ast) = parse_data.program {
        let (registry, mut diagnostics) = Analyzer::new(ast).analyze(&lib);
        all_diagnostics.append(&mut diagnostics);
    }
    for item in &all_diagnostics {
        println!("{}\n", item.render(&parse_data.src));
    }
}
