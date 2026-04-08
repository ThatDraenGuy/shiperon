use crate::analyzer::registry::{ClassId, ConsId};

pub enum PrimitiveExpr {
    Integer(i32),
    Real(f32),
    String(String),
    Char(char),
}

pub struct ConsExpr {
    pub class: ClassId,
    pub cons: ConsId,
    pub args: Vec<Expr>,
}

pub enum Expr {}

pub struct ExprModel {
    pub expr_type: ClassId,
    pub expr: Expr,
}

impl ExprModel {}
