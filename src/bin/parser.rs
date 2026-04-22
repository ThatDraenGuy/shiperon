use std::{error::Error, fs::read_to_string};

use ron::ser::PrettyConfig;
use shiperon::{CompilerConfig, Lexer, Parser, diagnostics::Renderable};

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_to_string("src/analyzer/std.po")?;
    let parser = Parser::new(Lexer::of_str(&input), CompilerConfig::default());
    let parse_data = parser.consume_parse();
    let str_result = ron::ser::to_string_pretty(&parse_data.program, PrettyConfig::default())?;
    println!("{str_result}");
    for item in &parse_data.diagnostics {
        println!("{}\n", item.render(&parse_data.src));
    }
    Ok(())
}
