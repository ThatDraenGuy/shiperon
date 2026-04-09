use std::rc::Rc;

use crate::{
    analyzer::{
        AnalysisError,
        body::ScopeStack,
        field::ClassWithFieldRegistry,
        registry::{ClassId, ConsId, FieldId, MethodId},
    },
    ast::{ShipArgs, ShipExprAll},
};

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

pub enum Expr {
    Varaible,
    FieldRead { class: ClassId, field: FieldId },
    ConsCall { class: ClassId, cons: ConsId, args: Vec<ExprModel> },
    MethodCall { class: ClassId, method: MethodId, args: Vec<ExprModel> },
    Primitive(PrimitiveExpr),
    Invalid,
}

pub struct ExprModel {
    pub expr_type: ClassId,
    pub expr: Expr,
}

impl ExprModel {
    pub fn resolve<'src>(
        registry: &ClassWithFieldRegistry<'src>,
        ctx: &ScopeStack<'src>,
        expr: &ShipExprAll<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        match expr {
            ShipExprAll::MemberAccess(node) => todo!(),
            ShipExprAll::Call(node) => todo!(),
            ShipExprAll::Primary(ship_primary_all) => todo!(),
            ShipExprAll::ClassCast(node) => todo!(),
        }
    }

    pub fn resolve_args<'src>(
        registry: &ClassWithFieldRegistry<'src>,
        ctx: &ScopeStack<'src>,
        args: &Rc<ShipArgs<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Vec<ExprModel> {
        args.exprs.iter().map(|expr| Self::resolve(registry, ctx, expr, errors)).collect()
    }
}
