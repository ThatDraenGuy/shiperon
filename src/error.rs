use crate::parser;

#[derive(thiserror::Error, Debug)]
pub enum ShipError {
    #[error(transparent)]
    Parser(#[from] parser::ParserError),
}
