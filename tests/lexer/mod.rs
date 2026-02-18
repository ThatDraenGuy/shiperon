use std::{error::Error, path::Path};

use insta::{assert_snapshot, glob};
use shiperon::{Lexer, TokenRegistry, lexer::Token};

fn perform_test(input_path: &Path) -> Result<String, Box<dyn Error>> {
    // let input = read(input_path)?;

    let mut lexer = Lexer::of_file(input_path)?;
    let mut result: Vec<Token> = Vec::new();
    loop {
        let token = lexer.yylex();
        let token_type = token.token_type;
        result.push(token);
        if token_type == TokenRegistry::YYEOF {
            break;
        }
    }
    Ok(result.iter().map(|t| t.to_string()).collect::<Vec<String>>().join("\n"))
}

#[test]
fn integration() {
    glob!("inputs/*.po", |input_path| {
        let result = perform_test(input_path).unwrap();
        assert_snapshot!(result);
    })
}
