use crate::parser::{ParserLoc, parse::SymbolKind};

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("unexpected token {token:?} at {loc:?}")]
    UnexpectedToken { token: SymbolKind, loc: ParserLoc },
}
