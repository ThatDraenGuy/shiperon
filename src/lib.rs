pub mod analyzer;
pub mod ast;

pub mod config;
use std::rc::Rc;

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

use crate::analyzer::Analyzer;
use crate::analyzer::stdlib::stdlib;
use crate::diagnostics::Renderable;

pub fn process(input: &str, config: CompilerConfig) {
    let parser = Parser::new(Lexer::of_str(input), config);
    let parse_data = parser.consume_parse();

    let mut all_diagnostics = parse_data.diagnostics;
    let lib = Rc::new(stdlib());
    if let Some(ast) = parse_data.program {
        let (registry, mut diagnostics) = Analyzer::new(ast).analyze(lib);
        all_diagnostics.append(&mut diagnostics);
    }
    for item in &all_diagnostics {
        println!("{}\n", item.render(&parse_data.src));
    }
}
