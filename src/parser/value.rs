use crate::lexer::Token;

#[derive(Clone, Debug, Default)]
pub enum ParserValue {
    None,
    Uninitialized,
    #[default]
    Stolen,
    Token(Token),
}

impl Token {
    pub fn from(value: ParserValue) -> Token {
        match value {
            ParserValue::Token(t) => t,
            other => unreachable!("expected Token, got {:?}", other),
        }
    }
}

impl ParserValue {
    /// Required method, parser expects it to be defined.
    ///
    /// Constructor for `Value::Token(token)` variant.
    pub fn from_token(value: Token) -> Self {
        Self::Token(value)
    }

    pub fn new_uninitialized() -> Self {
        Self::Uninitialized
    }

    pub fn is_uninitialized(&self) -> bool {
        matches!(self, Self::Uninitialized)
    }
}
