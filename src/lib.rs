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

use crate::analyzer::Analyzer;
use crate::analyzer::model::ClassModelRegistry;
use crate::diagnostics::Diagnostic;
use crate::source::StrSource;

pub fn process<'src>(
    input: &'src str,
    config: CompilerConfig,
) -> (Option<ClassModelRegistry>, Vec<Diagnostic<'src>>, StrSource<'src>) {
    let parser = Parser::new(Lexer::of_str(input), config);
    let parse_data = parser.consume_parse();

    let mut all_diagnostics = parse_data.diagnostics;
    let result = parse_data.program.map(|ast| {
        let (registry, mut diagnostics) = Analyzer::new(ast).analyze();
        all_diagnostics.append(&mut diagnostics);
        registry
    });
    (result, all_diagnostics, parse_data.src)
}
