use crate::lexer::Token;

#[derive(Clone, Debug, Default)]
pub enum ParserValue {
    None,
    Uninitialized,
    #[default]
    Stolen,
    Token(Token),
}

impl ParserValue {
    /// Required method, parser expects it to be defined.
    ///
    /// Constructor for `Value::Token(token)` variant.
    pub(crate) fn from_token(value: Token) -> Self {
        Self::Token(value)
    }

    pub(crate) fn new_uninitialized() -> Self {
        Self::Uninitialized
    }

    pub(crate) fn is_uninitialized(&self) -> bool {
        matches!(self, Self::Uninitialized)
    }
}
