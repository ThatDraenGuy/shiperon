use std::rc::Rc;

use crate::{
    analyzer::{
        AnalysisError,
        body::{AssignTarget, BodyError, ScopeStack, ScopeVar},
        def::ClassMemberNamesRegistry,
        field::{ClassFieldsRegistry, FieldModel},
        registry::{ClassId, ClassNameRegistry, ConsId, FieldId, LibClassId, MethodId, VarId},
        signature::ClassSignatureRegistry,
        stdlib::ShipStdLib,
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
    Method { class: ClassId, method: MethodId, args: Vec<ExprModel> },
    Invalid,
}

pub enum Expr {
    Varaible(VarId),
    FieldRead { expr: Box<ExprModel>, field: FieldId },
    Call(CallExpr),
    Primitive(PrimitiveExpr),
    This,
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

impl ExprModel {
    pub fn resolve_callable<'src>(
        stdlib: &ShipStdLib,
        cls_names: &ClassNameRegistry<'src>,
        member_names: &ClassMemberNamesRegistry<'src>,
        signatures: &ClassSignatureRegistry,
        fields: &ClassFieldsRegistry,
        call: &Rc<ShipCallExpr<'src>>,
        ctx: &ScopeStack<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> (Option<ClassId>, CallExpr) {
        let resolve_method_call =
            |object_type: ClassId,
             method_name: &Rc<ShipId<'src>>,
             method_args: &Rc<ShipArgs<'src>>,
             errors: &mut Vec<AnalysisError<'src>>| {
                let args = ExprModel::resolve_args(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    method_args,
                    errors,
                );
                let arg_types: Vec<_> = args.iter().map(|arg| arg.expr_type).collect();

                signatures
                    .find_matching_method(
                        stdlib,
                        member_names,
                        object_type,
                        method_name.id,
                        &arg_types,
                        method_name,
                        method_args,
                    )
                    .map(|(cls_id, method_id, signature)| {
                        (
                            signature.return_type,
                            CallExpr::Method { class: cls_id, method: method_id, args },
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
                let args = ExprModel::resolve_args(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    cons_args,
                    errors,
                );
                let arg_types: Vec<_> = args.iter().map(|arg| arg.expr_type).collect();

                signatures
                    .get_cls_signature(stdlib, &cls_id)
                    .constructors
                    .find_matching_cons(stdlib, &arg_types, &signatures, cons_args)
                    .map(|(cons_id, _cons_data)| {
                        (cls_id, CallExpr::Cons { class: ctx.curr_cls.into(), cons: cons_id, args })
                    })
                    .unwrap_or_else(|e| {
                        errors.push(e.into());
                        (cls_id, CallExpr::Invalid)
                    })
            };

        match &call.expr {
            ShipCallableExprAll::MemberAccess(member_access) => {
                let expr = ExprModel::resolve(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    &member_access.expr,
                    errors,
                );
                resolve_method_call(expr.expr_type, &member_access.member_id, &call.args, errors)
            },
            ShipCallableExprAll::This(_) => {
                let (cls_id, expr) = resolve_cons_call(ctx.curr_cls.into(), &call.args, errors);
                (Some(cls_id), expr)
            },
            ShipCallableExprAll::Cons(id_node) => {
                if let Ok(cls_id) = cls_names.get_class(id_node) {
                    let (cls_id, expr) = resolve_cons_call(cls_id, &call.args, errors);
                    (Some(cls_id), expr)
                } else {
                    resolve_method_call(ctx.curr_cls.into(), id_node, &call.args, errors)
                }
            },
            ShipCallableExprAll::Super(node) => todo!(),
        }
    }
    pub fn resolve_assignable<'src>(
        stdlib: &ShipStdLib,
        cls_names: &ClassNameRegistry<'src>,
        member_names: &ClassMemberNamesRegistry<'src>,
        signatures: &ClassSignatureRegistry,
        fields: &ClassFieldsRegistry,
        target: &ShipAssignableExprAll<'src>,
        ctx: &ScopeStack<'src>,
        value_type: ClassId,
        value_node: &ShipExprAll<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> AssignTarget {
        let resolve_field_assign = |field_id: FieldId,
                                    field_model: &FieldModel,
                                    target_object: ExprModel| {
            if !signatures.is_cls_subcls_of(stdlib, value_type, field_model.field_type).0 {
                Err(BodyError::TypeMismatch { expr: value_node.clone() })
            } else if !signatures.is_cls_subcls_of(stdlib, ctx.curr_cls, target_object.expr_type).0
            {
                Err(BodyError::AssignToExternalField { assign: target.clone() })
            } else {
                Ok(AssignTarget::Field(target_object, field_id))
            }
        };

        match &target {
            ShipAssignableExprAll::MemberAccess(member_access) => {
                let target_object = ExprModel::resolve(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    &member_access.expr,
                    errors,
                );
                match fields.find_field(
                    stdlib,
                    signatures,
                    member_names,
                    target_object.expr_type,
                    &member_access.member_id,
                ) {
                    Ok((field_id, field_model)) => {
                        resolve_field_assign(field_id, field_model, target_object).unwrap_or_else(
                            |e| {
                                errors.push(e.into());
                                AssignTarget::Invalid
                            },
                        )
                    },
                    Err(e) => {
                        errors.push(e.into());
                        AssignTarget::Invalid
                    },
                }
            },
            ShipAssignableExprAll::Variable(var_name) => {
                match ctx.find_var(stdlib, member_names, signatures, fields, var_name) {
                    Some(ScopeVar::Var(var_id, var_signature)) => {
                        if !signatures
                            .is_cls_subcls_of(stdlib, value_type, var_signature.var_type)
                            .0
                        {
                            errors
                                .push(BodyError::TypeMismatch { expr: value_node.clone() }.into());
                            AssignTarget::Invalid
                        } else if !var_signature.mutable {
                            errors.push(BodyError::AssignToConst { assign: target.clone() }.into());
                            AssignTarget::Invalid
                        } else {
                            AssignTarget::Var(var_id)
                        }
                    },
                    Some(ScopeVar::Field(field_id, field_model)) => resolve_field_assign(
                        field_id,
                        field_model,
                        ExprModel { expr_type: ctx.curr_cls.into(), expr: Expr::This },
                    )
                    .unwrap_or_else(|e| {
                        errors.push(e.into());
                        AssignTarget::Invalid
                    }),
                    Some(ScopeVar::Global) => todo!(),
                    None => {
                        errors.push(BodyError::UndefinedVariable { name: var_name.clone() }.into());
                        AssignTarget::Invalid
                    },
                }
            },
        }
    }

    pub fn resolve<'src>(
        stdlib: &ShipStdLib,
        cls_names: &ClassNameRegistry<'src>,
        member_names: &ClassMemberNamesRegistry<'src>,
        signatures: &ClassSignatureRegistry,
        fields: &ClassFieldsRegistry,
        ctx: &ScopeStack<'src>,
        expr: &ShipExprAll<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        match expr {
            ShipExprAll::MemberAccess(member_access) => {
                let expr = ExprModel::resolve(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    &member_access.expr,
                    errors,
                );
                match fields.find_field(
                    stdlib,
                    signatures,
                    member_names,
                    expr.expr_type,
                    &member_access.member_id,
                ) {
                    Ok((field_id, field_model)) => Self {
                        expr_type: field_model.field_type,
                        expr: Expr::FieldRead { expr: expr.into(), field: field_id },
                    },
                    Err(e) => {
                        errors.push(e.into());
                        Self { expr_type: ClassId::Invalid, expr: Expr::Invalid }
                    },
                }
            },
            ShipExprAll::Call(call) => {
                let (return_type, call_expr) = Self::resolve_callable(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    call,
                    ctx,
                    errors,
                );
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
                    Self { expr_type: ctx.curr_cls.into(), expr: Expr::This }
                },
                ShipPrimaryAll::Id(id_node) => {
                    match ctx.find_var(stdlib, member_names, signatures, fields, id_node) {
                        Some(ScopeVar::Var(var_id, var_signature)) => {
                            Self { expr_type: var_signature.var_type, expr: Expr::Varaible(var_id) }
                        },
                        Some(ScopeVar::Field(field_id, field_model)) => Self {
                            expr_type: field_model.field_type,
                            expr: Expr::FieldRead {
                                expr: ExprModel {
                                    expr_type: ctx.curr_cls.into(),
                                    expr: Expr::This,
                                }
                                .into(),
                                field: field_id,
                            },
                        },
                        Some(ScopeVar::Global) => todo!(),
                        None => {
                            errors.push(
                                BodyError::UndefinedVariable { name: id_node.clone() }.into(),
                            );
                            Self { expr_type: ClassId::Invalid, expr: Expr::Invalid }
                        },
                    }
                },
            },
            ShipExprAll::ClassCast(node) => todo!(),
        }
    }

    pub fn resolve_args<'src>(
        stdlib: &ShipStdLib,
        cls_names: &ClassNameRegistry<'src>,
        member_names: &ClassMemberNamesRegistry<'src>,
        signatures: &ClassSignatureRegistry,
        fields: &ClassFieldsRegistry,
        ctx: &ScopeStack<'src>,
        args: &Rc<ShipArgs<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Vec<ExprModel> {
        args.exprs
            .iter()
            .map(|expr| {
                Self::resolve(
                    stdlib,
                    cls_names,
                    member_names,
                    signatures,
                    fields,
                    ctx,
                    expr,
                    errors,
                )
            })
            .collect()
    }
}
