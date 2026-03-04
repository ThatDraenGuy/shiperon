use crate::{
    ast::*,
    lexer::{Token, TokenValue},
};

#[derive(Clone, Debug, Default)]
pub enum ParserValue {
    None,
    Uninitialized,
    #[default]
    Stolen,
    Token(Token),
    Primary(ShipPrimary),
    Expr(ShipExpression),
    Args(Vec<ShipExpression>),
}

impl Token {
    pub fn from(value: ParserValue) -> Token {
        match value {
            ParserValue::Token(t) => t,
            other => unreachable!("expected Token, got {:?}", other),
        }
    }
}

impl ShipPrimary {
    pub fn from(value: ParserValue) -> ShipPrimary {
        match value {
            ParserValue::Primary(p) => p,
            other => unreachable!("expected Primary, got {:?}", other),
        }
    }
}

impl ShipExpression {
    pub fn from(value: ParserValue) -> ShipExpression {
        match value {
            ParserValue::Expr(e) => e,
            other => unreachable!("expected Expr, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod Args {
    use super::ParserValue;
    use crate::ast::ShipExpression;

    pub fn from(value: ParserValue) -> Vec<ShipExpression> {
        match value {
            ParserValue::Args(a) => a,
            other => unreachable!("expected Args, got {:?}", other),
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

    pub fn new_primary_int(token: Token) -> Self {
        match token.token_value {
            TokenValue::Int(i) => Self::Primary(ShipPrimary::Int(i)),
            other => unreachable!("expected Integer, got {:?}", other),
        }
    }

    pub fn new_primary_float(token: Token) -> Self {
        match token.token_value {
            TokenValue::Float(f) => Self::Primary(ShipPrimary::Float(f)),
            other => unreachable!("expected Integer, got {:?}", other),
        }
    }

    pub fn new_primary_this() -> Self {
        Self::Primary(ShipPrimary::This)
    }

    pub fn new_expr_primary(primary: ShipPrimary) -> Self {
        Self::Expr(ShipExpression::Primary(primary))
    }

    pub fn new_expr_member_access(object: ShipExpression, token: Token) -> Self {
        match token.token_value {
            TokenValue::String(s) => {
                Self::Expr(ShipExpression::MemberAccess { object: Box::new(object), member_id: s })
            },
            other => unreachable!("expected Integer, got {:?}", other),
        }
    }

    pub fn new_args() -> Self {
        Self::Args(Vec::new())
    }
}
