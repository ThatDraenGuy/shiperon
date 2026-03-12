use std::{error::Error, fs::read_to_string};

use ron::ser::PrettyConfig;
use shiperon::{Lexer, Parser};

fn main() -> Result<(), Box<dyn Error>> {
    let input = read_to_string("tests/parser/inputs/invalid.po")?;
    let parser = Parser::new(Lexer::of_str(&input), false);
    let parse_data = parser.consume_parse();
    let str_result = ron::ser::to_string_pretty(&parse_data.program, PrettyConfig::default())?;
    println!("{str_result}");
    for item in &parse_data.diagnostics {
        println!("{}\n", item.render(&parse_data.src));
    }
    Ok(())
}
