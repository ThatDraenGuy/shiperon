use std::fmt::{Debug, Display};

use crate::{
    Lexer,
    parser::{ParserLoc, token_name},
};

#[derive(Clone)]
pub struct Token {
    pub token_type: i32,
    pub token_value: Vec<u8>,
    pub loc: ParserLoc,
}

impl Token {
    pub fn type_name(&self) -> &'static str {
        if self.token_type == Lexer::YYUNDEF {
            "YYUNDEF"
        } else if self.token_type == Lexer::YYerror {
            "YYerror"
        } else {
            token_name(self.token_type)
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "[ ({:0>3}..{:0>3})\t{} {}]",
            self.loc.begin,
            self.loc.end,
            self.type_name(),
            str::from_utf8(&self.token_value).unwrap_or("invalid value"),
        ))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}
