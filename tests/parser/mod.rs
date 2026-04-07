use std::{error::Error, fs::read_to_string, path::Path};

use insta::{assert_snapshot, glob};
use ron::ser::PrettyConfig;
use serde::Serialize;
use shiperon::{CompilerConfig, Lexer, Parser, diagnostics::Renderable};

#[derive(Debug, Serialize)]
struct TestOutput {
    pub ast: String,
    pub diagnostics: String,
}

fn perform_test(input_path: &Path) -> Result<String, Box<dyn Error>> {
    let input = read_to_string(input_path)?;

    let parser = Parser::new(Lexer::of_str(&input), CompilerConfig::default());
    let result = parser.consume_parse();
    let output = TestOutput {
        ast: ron::ser::to_string_pretty(&result.program, PrettyConfig::default())?,
        diagnostics: result
            .diagnostics
            .iter()
            .map(|d| d.render(&result.src))
            .collect::<Vec<String>>()
            .join("\n\n"),
    };
    Ok(serde_yaml::to_string(&output)?)
}

#[test]
fn integration() {
    glob!("inputs/*.po", |input_path| {
        let result = perform_test(input_path).unwrap();
        assert_snapshot!(result);
    })
}
