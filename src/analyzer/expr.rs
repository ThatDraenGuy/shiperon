use std::rc::Rc;

use crate::{
    StdlibCtx,
    analyzer::{
        AnalysisError,
        body::{AssignTarget, BodyError, ScopeStack, ScopeVar},
        def::{ClassMemberNamesCtx, ClassNamesCtx},
        field::{ClassFieldsCtx, FieldModel, FindFieldCtx},
        registry::{ClassId, ConsId, FieldId, LibClassId, MethodId, VarId},
        signature::{ClassSignatureCtx, FindMatchingMethodCtx, GetClsSignatureCtx},
    },
    ast::{
        ShipArgs, ShipAssignableExprAll, ShipCallExpr, ShipCallableExprAll, ShipExprAll, ShipId,
        ShipPrimaryAll,
    },
};

pub enum PrimitiveExpr {
    Integer(i32),
    Real(f32),
    String(String),
    Char(char),
}

pub enum CallExpr {
    Cons { class: ClassId, cons: ConsId, args: Vec<ExprModel> },
    Method { object: Box<ExprModel>, class: ClassId, method: MethodId, args: Vec<ExprModel> },
    Invalid,
}

pub enum Expr {
    Variable(VarId, usize),
    FieldRead { expr: Box<ExprModel>, owner_cls: ClassId, field: FieldId },
    Call(CallExpr),
    Primitive(PrimitiveExpr),
    This,
    ClassCast { expr: Box<ExprModel>, cls_id: ClassId },
    Invalid,
}
impl From<PrimitiveExpr> for Expr {
    fn from(value: PrimitiveExpr) -> Self {
        Self::Primitive(value)
    }
}

pub struct ExprModel {
    pub expr_type: ClassId,
    pub expr: Expr,
}

pub trait ExprResolutionCtx<'src>:
    StdlibCtx + ClassNamesCtx<'src> + ClassMemberNamesCtx<'src> + ClassSignatureCtx + ClassFieldsCtx
{
}
impl<
    'src,
    Ctx: StdlibCtx
        + ClassNamesCtx<'src>
        + ClassMemberNamesCtx<'src>
        + ClassSignatureCtx
        + ClassFieldsCtx,
> ExprResolutionCtx<'src> for Ctx
{
}

impl ExprModel {
    pub fn resolve_callable<'src>(
        ctx: &impl ExprResolutionCtx<'src>,
        scopes: &ScopeStack<'src>,
        call: &Rc<ShipCallExpr<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> (Option<ClassId>, CallExpr) {
        let resolve_method_call =
            |object: ExprModel,
             method_name: &Rc<ShipId<'src>>,
             method_args: &Rc<ShipArgs<'src>>,
             errors: &mut Vec<AnalysisError<'src>>| {
                let args = ExprModel::resolve_args(ctx, scopes, method_args, errors);
                let arg_types: Vec<_> = args.iter().map(|arg| arg.expr_type).collect();

                ctx.find_matching_method(
                    object.expr_type,
                    method_name.id,
                    &arg_types,
                    method_name,
                    method_args,
                )
                .map(|(cls_id, method_id, signature)| {
                    let (cls_id, method_id) = ctx.get_top_method(cls_id, method_id);
                    (
                        signature.return_type,
                        CallExpr::Method {
                            object: object.into(),
                            class: cls_id,
                            method: method_id,
                            args,
                        },
                    )
                })
                .unwrap_or_else(|e| {
                    errors.push(e.into());
                    (Some(ClassId::Invalid), CallExpr::Invalid)
                })
            };
        let resolve_cons_call =
            |cls_id: ClassId,
             cons_args: &Rc<ShipArgs<'src>>,
             errors: &mut Vec<AnalysisError<'src>>| {
                let args = ExprModel::resolve_args(ctx, scopes, cons_args, errors);
                let arg_types: Vec<_> = args.iter().map(|arg| arg.expr_type).collect();

                ctx.get_cls_signature(&cls_id)
                    .constructors
                    .find_matching_cons(ctx, &arg_types, cons_args)
                    .map(|(cons_id, _cons_data)| {
                        (
                            cls_id,
                            CallExpr::Cons { class: scopes.curr_cls.into(), cons: cons_id, args },
                        )
                    })
                    .unwrap_or_else(|e| {
                        errors.push(e.into());
                        (cls_id, CallExpr::Invalid)
                    })
            };

        match &call.expr {
            ShipCallableExprAll::MemberAccess(member_access) => {
                let expr = ExprModel::resolve(ctx, scopes, &member_access.expr, errors);
                resolve_method_call(expr, &member_access.member_id, &call.args, errors)
            },
            ShipCallableExprAll::This(_) => {
                let (cls_id, expr) = resolve_cons_call(scopes.curr_cls.into(), &call.args, errors);
                (Some(cls_id), expr)
            },
            ShipCallableExprAll::Cons(id_node) => {
                if let Ok(cls_id) = ctx.cls_names().get_class(id_node) {
                    let (cls_id, expr) = resolve_cons_call(cls_id, &call.args, errors);
                    (Some(cls_id), expr)
                } else {
                    resolve_method_call(
                        ExprModel { expr_type: scopes.curr_cls.into(), expr: Expr::This },
                        id_node,
                        &call.args,
                        errors,
                    )
                }
            },
            ShipCallableExprAll::Super(_node) => unimplemented!(),
        }
    }
    pub fn resolve_assignable<'src>(
        ctx: &impl ExprResolutionCtx<'src>,
        scopes: &ScopeStack<'src>,
        target: &ShipAssignableExprAll<'src>,
        value_type: ClassId,
        value_node: &ShipExprAll<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> AssignTarget {
        let resolve_field_assign = |cls_id: ClassId,
                                    field_id: FieldId,
                                    field_model: &FieldModel,
                                    target_object: ExprModel| {
            if !ctx.is_cls_subcls_of(value_type, field_model.field_type).0 {
                Err(BodyError::TypeMismatch { expr: value_node.clone() })
            } else if !ctx.is_cls_subcls_of(scopes.curr_cls, target_object.expr_type).0 {
                Err(BodyError::AssignToExternalField { assign: target.clone() })
            } else {
                Ok(AssignTarget::Field(target_object, cls_id, field_id))
            }
        };

        match &target {
            ShipAssignableExprAll::MemberAccess(member_access) => {
                let target_object = ExprModel::resolve(ctx, scopes, &member_access.expr, errors);
                match ctx.find_field(target_object.expr_type, &member_access.member_id) {
                    Ok((cls_id, field_id, field_model)) => {
                        resolve_field_assign(cls_id, field_id, field_model, target_object)
                            .unwrap_or_else(|e| {
                                errors.push(e.into());
                                AssignTarget::Invalid
                            })
                    },
                    Err(e) => {
                        errors.push(e.into());
                        AssignTarget::Invalid
                    },
                }
            },
            ShipAssignableExprAll::Variable(var_name) => match scopes.find_var(ctx, var_name) {
                Some(ScopeVar::Var(var_id, offset, var_signature)) => {
                    if !ctx.is_cls_subcls_of(value_type, var_signature.var_type).0 {
                        errors.push(BodyError::TypeMismatch { expr: value_node.clone() }.into());
                        AssignTarget::Invalid
                    } else if !var_signature.mutable {
                        errors.push(BodyError::AssignToConst { assign: target.clone() }.into());
                        AssignTarget::Invalid
                    } else {
                        AssignTarget::Var(var_id, offset)
                    }
                },
                Some(ScopeVar::Field(cls_id, field_id, field_model)) => resolve_field_assign(
                    cls_id,
                    field_id,
                    field_model,
                    ExprModel { expr_type: scopes.curr_cls.into(), expr: Expr::This },
                )
                .unwrap_or_else(|e| {
                    errors.push(e.into());
                    AssignTarget::Invalid
                }),
                Some(ScopeVar::Global) => {
                    errors.push(BodyError::AssignToConst { assign: target.clone() }.into());
                    AssignTarget::Invalid
                },
                None => {
                    errors.push(BodyError::UndefinedVariable { name: var_name.clone() }.into());
                    AssignTarget::Invalid
                },
            },
        }
    }

    pub fn resolve<'src>(
        ctx: &impl ExprResolutionCtx<'src>,
        scopes: &ScopeStack<'src>,
        expr: &ShipExprAll<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        match expr {
            ShipExprAll::MemberAccess(member_access) => {
                let expr = ExprModel::resolve(ctx, scopes, &member_access.expr, errors);
                match ctx.find_field(expr.expr_type, &member_access.member_id) {
                    Ok((cls_id, field_id, field_model)) => Self {
                        expr_type: field_model.field_type,
                        expr: Expr::FieldRead {
                            expr: expr.into(),
                            owner_cls: cls_id,
                            field: field_id,
                        },
                    },
                    Err(e) => {
                        errors.push(e.into());
                        Self { expr_type: ClassId::Invalid, expr: Expr::Invalid }
                    },
                }
            },
            ShipExprAll::Call(call) => {
                let (return_type, call_expr) = Self::resolve_callable(ctx, scopes, call, errors);
                match return_type {
                    Some(return_type) => {
                        Self { expr_type: return_type, expr: Expr::Call(call_expr) }
                    },
                    None => {
                        errors.push(
                            BodyError::InvalidVoidReturningCall { call: call.clone() }.into(),
                        );
                        Self { expr_type: ClassId::Invalid, expr: Expr::Call(call_expr) }
                    },
                }
            },
            ShipExprAll::Primary(primary) => match primary {
                ShipPrimaryAll::Int(int_node) => Self {
                    expr_type: LibClassId::Integer.into(),
                    expr: PrimitiveExpr::Integer(int_node.int).into(),
                },
                ShipPrimaryAll::Float(float_node) => Self {
                    expr_type: LibClassId::Real.into(),
                    expr: PrimitiveExpr::Real(float_node.float).into(),
                },
                ShipPrimaryAll::String(string_node) => Self {
                    expr_type: LibClassId::String.into(),
                    expr: PrimitiveExpr::String(string_node.string.clone()).into(),
                },
                ShipPrimaryAll::Char(char_node) => Self {
                    expr_type: LibClassId::Char.into(),
                    expr: PrimitiveExpr::Char(char_node.char).into(),
                },
                ShipPrimaryAll::This(_node) => {
                    Self { expr_type: scopes.curr_cls.into(), expr: Expr::This }
                },
                ShipPrimaryAll::Id(id_node) => match scopes.find_var(ctx, id_node) {
                    Some(ScopeVar::Var(var_id, offset, var_signature)) => Self {
                        expr_type: var_signature.var_type,
                        expr: Expr::Variable(var_id, offset),
                    },
                    Some(ScopeVar::Field(cls_id, field_id, field_model)) => Self {
                        expr_type: field_model.field_type,
                        expr: Expr::FieldRead {
                            expr: ExprModel { expr_type: scopes.curr_cls.into(), expr: Expr::This }
                                .into(),
                            owner_cls: cls_id,
                            field: field_id,
                        },
                    },
                    Some(ScopeVar::Global) => unimplemented!(),
                    None => {
                        errors.push(BodyError::UndefinedVariable { name: id_node.clone() }.into());
                        Self { expr_type: ClassId::Invalid, expr: Expr::Invalid }
                    },
                },
            },
            ShipExprAll::ClassCast(class_cast) => {
                let inner = ExprModel::resolve(ctx, scopes, &class_cast.expr, errors);
                let cast_cls = ctx.cls_names().get_class_with_err(&class_cast.class_id, errors);
                if !ctx.is_cls_subcls_of(cast_cls, inner.expr_type).0 {
                    errors.push(BodyError::InvalidClassCast { cast: class_cast.clone() }.into());
                    Self { expr_type: ClassId::Invalid, expr: Expr::Invalid }
                } else {
                    Self {
                        expr_type: cast_cls,
                        expr: Expr::ClassCast { expr: inner.into(), cls_id: cast_cls },
                    }
                }
            },
        }
    }

    pub fn resolve_args<'src>(
        ctx: &impl ExprResolutionCtx<'src>,
        scopes: &ScopeStack<'src>,
        args: &Rc<ShipArgs<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Vec<ExprModel> {
        args.exprs.iter().map(|expr| Self::resolve(ctx, scopes, expr, errors)).collect()
    }
}
