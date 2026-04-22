#[allow(warnings)] // codegen file
mod parse;
use std::rc::Rc;

use derive_more::Display;
pub use parse::{Parser, token_name};

mod value;
pub use value::ParserValue;

mod loc;
pub use loc::{ParserLoc, WithParserLoc};

use crate::{
    ByteSource, ShipFeature,
    ast::{ShipArgs, ShipExprAll, ShipProgram, ShipReturnStmt},
    diagnostics::{Diagnostic, Renderable},
};

pub struct ParseData<'src, S: ByteSource<'src>> {
    pub program: Option<Rc<ShipProgram<'src>>>,
    pub diagnostics: Vec<Diagnostic<'src>>,
    pub src: S,
}

#[derive(Debug, Clone, Display)]
pub enum ParseError<'src> {
    #[display("Unexpected token: {token_name}")]
    UnexpectedToken { token_name: &'static str },
    #[display("Body members after this return stmt: {return_stmt}")]
    BodyMembersAfterReturn { return_stmt: Rc<ShipReturnStmt<'src>> },
    #[display("Feature \"{}\" is disabled", _0.name())]
    DisabledFeature(ShipFeature),
    #[display("Expr is not callable, call args: {call_args}")]
    ExprIsNotCallable { call_args: Rc<ShipArgs<'src>> },
    #[display("Expr is not assignable")]
    ExprIsNotAssignable { value: ShipExprAll<'src> },
    #[display("Assign attempted instead of variable definition")]
    AssignOnVarDef,
    #[display("Parenthesis are unnecessary")]
    UnnecessaryParenthesis,
    #[display("Constructors don't have return types")]
    ReturnTypeInCons,
}

impl<'src> Renderable<'src> for ParseError<'src> {
    fn render(&self, _src: &impl ByteSource<'src>) -> String {
        match self {
            Self::UnexpectedToken { token_name: _ } => "Unexpected token".to_owned(),
            Self::BodyMembersAfterReturn { return_stmt } => format!(
                "Unreachable code after return statement. Return originally invoked here:\n{}:\n{}",
                return_stmt.start,
                str::from_utf8(return_stmt.src).unwrap_or("invalid utf-8 string")
            ),
            Self::DisabledFeature(feature) => {
                format!("Feature \"{}\" is disabled", feature.name())
            },
            Self::ExprIsNotCallable { call_args } => format!(
                "Expression is not callable, but a call is attempted with these args:\n{}\n{}",
                call_args.start,
                str::from_utf8(call_args.src).unwrap_or("invalid utf-8 string")
            ),
            Self::ExprIsNotAssignable { value: _value } => {
                "Expression is not assignable, but an assign is attempted".to_string()
            },
            Self::AssignOnVarDef => {
                "Assign attempted instead of variable definition; replace `:=` with `:`".to_owned()
            },
            Self::UnnecessaryParenthesis => "Parenthesis are unnecessary".to_owned(),
            Self::ReturnTypeInCons => "Constructors don't have return types".to_owned(),
        }
    }
}
