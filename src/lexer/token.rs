use std::fmt::{Debug, Display};

use crate::{
    TokenRegistry,
    parser::{ParserLoc, WithParserLoc, token_name},
};

pub type TokenType = i32;

#[derive(Clone)]
pub enum TokenValue {
    None,
    Int(i32),
    Float(f32),
    String(String),
}

impl Debug for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Int(i) => f.write_str(&format!("{i}")),
            Self::Float(fl) => f.write_str(&format!("{fl}")),
            Self::String(s) => f.write_str(s),
        }
    }
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub token_value: TokenValue,
    pub loc: ParserLoc,
}

impl Token {
    pub fn type_name(&self) -> &'static str {
        if self.token_type == TokenRegistry::YYUNDEF {
            "YYUNDEF"
        } else if self.token_type == TokenRegistry::YYerror {
            "YYerror"
        } else {
            token_name(self.token_type)
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[ ({:?})\t{} {}]", self.loc, self.type_name(), self.token_value))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

impl WithParserLoc for Token {
    fn loc(&self) -> ParserLoc {
        self.loc
    }
}
