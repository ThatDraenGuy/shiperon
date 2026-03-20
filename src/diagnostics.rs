use std::{
    fmt::{Display, format},
    rc::Rc,
};

use crate::{
    ByteSource, ShipFeature,
    ast::{ShipArgs, ShipExprAll, ShipReturnStmt},
    parser::ParserLoc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLevel {
    Warn,
    Err,
}

#[derive(Debug, Clone)]
pub struct Diagnostic<'src> {
    pub level: ErrorLevel,
    pub loc: ParserLoc,
    pub reason: Reason<'src>,
}

impl<'src> Diagnostic<'src> {
    pub fn render(&self, src: &impl ByteSource<'src>) -> String {
        let (start, _end) = src.resolve(self.loc);

        let view = str::from_utf8(src.source(self.loc)).unwrap_or("invalid utf-8 string");
        format!(
            "{} at {}:\n{}\n^\n{}",
            match self.level {
                ErrorLevel::Warn => "Warning",
                ErrorLevel::Err => "Error",
            },
            &start,
            view,
            self.reason.render(src)
        )
    }
}

impl<'src> Display for Diagnostic<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:?}] ({:?}) {}", self.level, self.loc, self.reason))
    }
}

#[derive(Debug, Clone)]
pub enum Reason<'src> {
    UnexpectedToken { token_name: &'static str },
    BodyMembersAfterReturn { return_stmt: Rc<ShipReturnStmt<'src>> },
    DisabledFeature(ShipFeature),
    ExprIsNotCallable { call_args: Rc<ShipArgs<'src>> },
    ExprIsNotAssignable { value: ShipExprAll<'src> },
}

impl<'src> Reason<'src> {
    fn render(&self, _src: &impl ByteSource<'src>) -> String {
        match self {
            Reason::UnexpectedToken { token_name: _ } => "Unexpected token".to_owned(),
            Reason::BodyMembersAfterReturn { return_stmt } => format!(
                "Unreachable code after return statement. Return originally invoked here:\n{}:\n{}",
                return_stmt.start,
                str::from_utf8(return_stmt.src).unwrap_or("invalid utf-8 string")
            ),
            Reason::DisabledFeature(feature) => {
                format!("Feature \"{}\" is disabled", feature.name())
            },
            Reason::ExprIsNotCallable { call_args } => format!(
                "Expression is not callable, but a call is attempted with these args:\n{}\n{}",
                call_args.start,
                str::from_utf8(call_args.src).unwrap_or("invalid utf-8 string")
            ),
            Reason::ExprIsNotAssignable { value: _value } => {
                "Expression is not assignable, but an assign is attempted".to_string()
            },
        }
    }
}

impl<'src> Display for Reason<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::UnexpectedToken { token_name } => {
                f.write_fmt(format_args!("Unexpected token: {token_name}",))
            },
            Reason::BodyMembersAfterReturn { return_stmt } => {
                f.write_fmt(format_args!("Body members after this return stmt: {return_stmt}"))
            },
            Reason::DisabledFeature(feature) => {
                f.write_fmt(format_args!("Feature \"{}\" is disabled", feature.name()))
            },
            Reason::ExprIsNotCallable { call_args } => {
                f.write_fmt(format_args!("Expr is not callable, call args: {call_args}"))
            },
            Reason::ExprIsNotAssignable { value: _value } => {
                f.write_fmt(format_args!("Expr is not assignable"))
            },
        }
    }
}
