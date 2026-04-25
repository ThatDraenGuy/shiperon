use std::{collections::LinkedList, rc::Rc};

use derive_more::Display;

use crate::{
    StdlibCtx,
    analyzer::{
        AnalysisError,
        def::{ClassMemberNamesCtx, ClassNamesCtx},
        expr::{CallExpr, Expr, ExprModel},
        field::{ClassFieldsCtx, FieldModel, FindFieldCtx},
        registry::{
            ClassId, ConsId, ConsRegistry, FieldId, LibClassId, MethodId, MethodRegistry, Registry,
            UserClassId, VarId, VarNameRegistryBuilder,
        },
        signature::{ClassSignatureCtx, GetClsSignatureCtx},
    },
    ast::*,
    diagnostics::Renderable,
    parser::{ParserLoc, WithParserLoc},
};

enum ScopeType {
    // Global,
    // Class(ClassId),
    Method(MethodId),
    Cons(ConsId),
    While,
    If,
    // Else,
}

pub struct VarSignature {
    pub var_type: ClassId,
    pub mutable: bool,
}

pub type VarSignatureRegistry<'src> = VarNameRegistryBuilder<'src, VarSignature>;
pub struct BodyScope<'src> {
    scope_type: ScopeType,
    vars: VarSignatureRegistry<'src>,
}

pub enum BodyReturn<T> {
    Void,
    Never,
    Value(T),
}

pub enum ScopeVar<'a> {
    Var(VarId, usize, &'a VarSignature),
    Field(ClassId, FieldId, &'a FieldModel),
    Global,
}

pub struct ScopeStack<'src> {
    inner: LinkedList<BodyScope<'src>>,
    pub curr_cls: UserClassId,
    pub expected_return: Option<Option<ClassId>>,
}

pub trait FindVarCtx<'src>:
    StdlibCtx + ClassMemberNamesCtx<'src> + ClassSignatureCtx + ClassFieldsCtx
{
}
impl<'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassSignatureCtx + ClassFieldsCtx>
    FindVarCtx<'src> for Ctx
{
}

impl<'src> ScopeStack<'src> {
    fn new_cons(cls: UserClassId, id: ConsId, params: VarSignatureRegistry<'src>) -> Self {
        let mut inner = LinkedList::new();
        inner.push_front(BodyScope { scope_type: ScopeType::Cons(id), vars: params });
        Self { inner, curr_cls: cls, expected_return: None }
    }
    fn new_method(
        cls: UserClassId,
        id: MethodId,
        params: VarSignatureRegistry<'src>,
        return_type: Option<ClassId>,
    ) -> Self {
        let mut inner = LinkedList::new();
        inner.push_front(BodyScope { scope_type: ScopeType::Method(id), vars: params });
        Self { inner, curr_cls: cls, expected_return: Some(return_type) }
    }

    fn enter(&mut self, scope_type: ScopeType) {
        self.inner.push_front(BodyScope { scope_type, vars: VarSignatureRegistry::default() });
    }

    fn exit(&mut self) -> BodyScope {
        self.inner.pop_front().unwrap()
    }

    pub fn curr(&self) -> &BodyScope<'src> {
        self.inner.front().unwrap()
    }

    pub fn curr_mut(&mut self) -> &mut BodyScope<'src> {
        self.inner.front_mut().unwrap()
    }

    pub fn find_var<'a>(
        &'a self,
        ctx: &'a impl FindVarCtx<'src>,
        name: &Rc<ShipId<'src>>,
    ) -> Option<ScopeVar<'a>> {
        for (num, scope) in self.inner.iter().enumerate() {
            if let Some(id) = scope.vars.names().get_by_name(name.id) {
                return Some(ScopeVar::Var(id, num, scope.vars.values().get(&id)));
            }
        }
        if let Ok((cls_id, field_id, field_model)) = ctx.find_field(self.curr_cls.into(), name) {
            return Some(ScopeVar::Field(cls_id, field_id, field_model));
        }
        None
    }
}

pub struct Body {
    pub vars: Registry<VarId, VarSignature>,
    pub stmts: Vec<Stmt>,
    pub return_expr: Option<BodyReturn<ExprModel>>,
}

pub trait BodyResolutionCtx<'src>:
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
> BodyResolutionCtx<'src> for Ctx
{
}

impl Body {
    fn resolve<'src>(
        ctx: &impl BodyResolutionCtx<'src>,
        scopes: &mut ScopeStack<'src>,
        body: &Rc<ShipBody<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let mut stmts = Vec::new();
        let mut has_unreachable = false;
        let mut unreachable_stmts = Vec::new();
        for member in &body.members {
            if has_unreachable {
                unreachable_stmts.push(member.clone());
            } else {
                stmts.push(match member {
                    ShipBodyMemberAll::VarDef(var_def) => {
                        let init_expr = ExprModel::resolve(ctx, scopes, &var_def.expr, errors);
                        let id = scopes.curr_mut().vars.update(var_def.var_id.id, |_maybe_old| {
                            VarSignature { var_type: init_expr.expr_type, mutable: true } //variable shadowing
                        });
                        Stmt::VarDef { id, init_expr }
                    },
                    ShipBodyMemberAll::Stmt(stmt) => match stmt {
                        ShipStmtAll::Assign(assign) => {
                            let value = ExprModel::resolve(ctx, scopes, &assign.value, errors);
                            let target = ExprModel::resolve_assignable(
                                ctx,
                                scopes,
                                &assign.target,
                                value.expr_type,
                                &assign.value,
                                errors,
                            );
                            Stmt::Assign(target, value)
                        },
                        ShipStmtAll::While(while_node) => {
                            let condition =
                                ExprModel::resolve(ctx, scopes, &while_node.condition, errors);
                            if condition.expr_type != LibClassId::Boolean.into() {
                                errors.push(
                                    BodyError::NonBoolCondition {
                                        condition: while_node.condition.clone(),
                                    }
                                    .into(),
                                );
                            }
                            scopes.enter(ScopeType::While);
                            let body = Self::resolve(ctx, scopes, &while_node.body, errors);
                            Stmt::While { condition, body }
                        },
                        ShipStmtAll::If(if_node) => {
                            let condition =
                                ExprModel::resolve(ctx, scopes, &if_node.condition, errors);
                            if condition.expr_type != LibClassId::Boolean.into() {
                                errors.push(
                                    BodyError::NonBoolCondition {
                                        condition: if_node.condition.clone(),
                                    }
                                    .into(),
                                );
                            }
                            scopes.enter(ScopeType::If);
                            let then_body = Self::resolve(ctx, scopes, &if_node.then_body, errors);
                            let else_body = if_node.else_body.as_ref().map(|else_body_node| {
                                scopes.enter(ScopeType::If);
                                let res = Self::resolve(ctx, scopes, else_body_node, errors);
                                res
                            });

                            if let Some(else_body) = &else_body
                                && then_body.return_expr.is_some()
                                && else_body.return_expr.is_some()
                            {
                                // both branches have return
                                has_unreachable = true;
                            }
                            Stmt::If { condition, then_body, else_body }
                        },
                        ShipStmtAll::Call(call_node) => {
                            let call_expr =
                                ExprModel::resolve_callable(ctx, scopes, call_node, errors);
                            Stmt::Call(call_expr.1)
                        },
                        ShipStmtAll::Return(return_stmt) => {
                            return Self {
                                vars: scopes.exit().vars.build().1,
                                stmts,
                                return_expr: match scopes.expected_return {
                                    // parser ensures no statements after return in bodies
                                    Some(return_type) => {
                                        Some(match (return_type, &return_stmt.value) {
                                            (None, None) => BodyReturn::Void,
                                            (Some(expected_type), Some(return_expr)) => {
                                                let expr = ExprModel::resolve(
                                                    ctx,
                                                    scopes,
                                                    return_expr,
                                                    errors,
                                                );
                                                if ctx
                                                    .is_cls_subcls_of(expr.expr_type, expected_type)
                                                    .0
                                                {
                                                    BodyReturn::Value(expr)
                                                } else {
                                                    errors.push(
                                                        BodyError::TypeMismatch {
                                                            expr: return_expr.clone(),
                                                        }
                                                        .into(),
                                                    );
                                                    BodyReturn::Value(ExprModel {
                                                        expr_type: ClassId::Invalid,
                                                        expr: expr.expr,
                                                    })
                                                }
                                            },
                                            (None, Some(_)) => {
                                                errors.push(
                                                    BodyError::ReturnValueInVoid {
                                                        return_stmt: return_stmt.clone(),
                                                    }
                                                    .into(),
                                                );
                                                BodyReturn::Void
                                            },
                                            (Some(expected_type), None) => {
                                                errors.push(
                                                    BodyError::VoidReturnInValued {
                                                        return_stmt: return_stmt.clone(),
                                                    }
                                                    .into(),
                                                );
                                                BodyReturn::Value(ExprModel {
                                                    expr_type: expected_type,
                                                    expr: Expr::Invalid,
                                                })
                                            },
                                        })
                                    },
                                    None => {
                                        errors.push(
                                            BodyError::ReturnInCons {
                                                return_stmt: return_stmt.clone(),
                                            }
                                            .into(),
                                        );
                                        None
                                    },
                                },
                            };
                        },
                    },
                });
            }
        }

        if has_unreachable {
            errors.push(BodyError::UnreachableStmts { stmts: unreachable_stmts }.into());
            Self { vars: scopes.exit().vars.build().1, stmts, return_expr: Some(BodyReturn::Never) }
        } else {
            Self { vars: scopes.exit().vars.build().1, stmts, return_expr: None }
        }
    }
}

pub enum AssignTarget {
    Var(VarId, usize),
    Field(ExprModel, ClassId, FieldId),
    Invalid,
}

pub enum Stmt {
    VarDef { id: VarId, init_expr: ExprModel },
    Assign(AssignTarget, ExprModel),
    While { condition: ExprModel, body: Body },
    If { condition: ExprModel, then_body: Body, else_body: Option<Body> },
    Call(CallExpr),
    Invalid,
}

pub struct ConsBody {
    pub body: Body,
}

impl ConsBody {
    pub fn resolve<'src>(
        ctx: &impl BodyResolutionCtx<'src>,
        cls_id: UserClassId,
        cons_id: ConsId,
        def: &Rc<ShipConsDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let signature = ctx.signatures().get(&cls_id).constructors.get(&cons_id);

        let params = signature.annotate_types(def.params.params.iter());
        let mut param_registry = VarSignatureRegistry::default();
        for (param_def, param_type) in params {
            if let Some((_old, _new)) = param_registry
                .insert(param_def.name.id, VarSignature { var_type: param_type, mutable: true })
            {
                errors.push(BodyError::DuplicateParam { snd: param_def.clone() }.into());
            }
        }

        let mut scopes = ScopeStack::new_cons(cls_id, cons_id, param_registry);
        let body = match &def.body {
            ShipConsBodyAll::Body(body) => Body::resolve(ctx, &mut scopes, body, errors),
            ShipConsBodyAll::Generated(_generated) => unreachable!(),
        };
        Self { body }
    }
}

pub type ConsBodyRegistry = ConsRegistry<ConsBody>;

pub struct MethodBody {
    pub body: Body,
}

impl MethodBody {
    pub fn resolve<'src>(
        ctx: &impl BodyResolutionCtx<'src>,
        cls_id: UserClassId,
        method_id: MethodId,
        def: &Rc<ShipMethodDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let signature = ctx.signatures().get(&cls_id).methods.get_method(&method_id);

        let params = signature.params.annotate_types(def.params.params.iter());
        let mut param_registry = VarSignatureRegistry::default();
        for (param_def, param_type) in params {
            if let Some((_old, _new)) = param_registry
                .insert(param_def.name.id, VarSignature { var_type: param_type, mutable: true })
            {
                errors.push(BodyError::DuplicateParam { snd: param_def.clone() }.into());
            }
        }

        let mut scopes =
            ScopeStack::new_method(cls_id, method_id, param_registry, signature.return_type);
        let body = match &def.body {
            Some(ShipMethodBodyAll::Expr(expr)) => {
                let expr_model = ExprModel::resolve(ctx, &scopes, expr, errors);
                Body {
                    vars: scopes.exit().vars.build().1,
                    stmts: vec![],
                    return_expr: Some(BodyReturn::Value(expr_model)),
                }
            },
            Some(ShipMethodBodyAll::Body(body)) => Body::resolve(ctx, &mut scopes, body, errors),
            Some(ShipMethodBodyAll::Generated(_generated)) => unreachable!(),
            None => {
                //skip for now - forward declaration not handled
                Body { vars: Registry::empty(), stmts: vec![], return_expr: None }
            },
        };
        Self { body }
    }
}

pub type MethodBodyRegistry = MethodRegistry<MethodBody>;

#[derive(Debug, Clone, Display)]
pub enum BodyError<'src> {
    #[display("return in cons")]
    ReturnInCons { return_stmt: Rc<ShipReturnStmt<'src>> },
    #[display("undefined variable")]
    UndefinedVariable { name: Rc<ShipId<'src>> },
    #[display("assign to a const variable")]
    AssignToConst { assign: ShipAssignableExprAll<'src> },
    #[display("assign into external class field")]
    AssignToExternalField { assign: ShipAssignableExprAll<'src> },
    #[display("assign with a wrong type")]
    TypeMismatch { expr: ShipExprAll<'src> },
    #[display("non boolean condition")]
    NonBoolCondition { condition: ShipExprAll<'src> },
    #[display("return with value in void method")]
    ReturnValueInVoid { return_stmt: Rc<ShipReturnStmt<'src>> },
    #[display("void return in non void method")]
    VoidReturnInValued { return_stmt: Rc<ShipReturnStmt<'src>> },
    #[display("unreachable code")]
    UnreachableStmts { stmts: Vec<ShipBodyMemberAll<'src>> },
    #[display("void returning method call in non void context")]
    InvalidVoidReturningCall { call: Rc<ShipCallExpr<'src>> },
    #[display("duplicate param")]
    DuplicateParam { snd: Rc<ShipParam<'src>> },
    #[display("invalid class cast")]
    InvalidClassCast { cast: Rc<ShipClassCastExpr<'src>> },
}
impl<'src> Renderable<'src> for BodyError<'src> {
    fn render(&self, _src: &impl crate::ByteSource<'src>) -> String {
        match self {
            BodyError::ReturnInCons { return_stmt: _ } => {
                "Return statement not allowed in a constructor".to_string()
            },
            BodyError::UndefinedVariable { name } => {
                format!("Variable with name `{}` was not found", name.id)
            },
            BodyError::AssignToConst { assign: _ } => {
                "Assign into a constant variable attempted".to_string()
            },
            BodyError::AssignToExternalField { assign: _ } => {
                "Fields of foreign classes are read-only".to_string()
            },
            BodyError::TypeMismatch { expr: _ } => "Type mismatch".to_string(),
            BodyError::NonBoolCondition { condition: _ } => {
                "Condition expression is nota boolean".to_string()
            },
            BodyError::ReturnValueInVoid { return_stmt: _ } => {
                "Return statement with value is not allowed in a void method".to_string()
            },
            BodyError::VoidReturnInValued { return_stmt: _ } => {
                "Return statement without value is not allowed in a non-void method".to_string()
            },
            BodyError::UnreachableStmts { stmts: _ } => "Dead code".to_string(),
            BodyError::InvalidVoidReturningCall { call: _ } => {
                "Attempted to use return value of a void method".to_string()
            },
            BodyError::DuplicateParam { snd } => format!(
                "Param with name `{}` is defined multiple times:\nSecond declaration is:\n{}\n{}",
                snd.name.id,
                snd.start,
                snd.src()
            ),
            BodyError::InvalidClassCast { cast: _cast } => {
                "Inner expression type is not a child of cast class".to_string()
            },
        }
    }
}
impl<'src> WithParserLoc for BodyError<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            BodyError::ReturnInCons { return_stmt } => return_stmt.loc(),
            BodyError::UndefinedVariable { name } => name.loc(),
            BodyError::AssignToConst { assign } => assign.loc(),
            BodyError::AssignToExternalField { assign } => assign.loc(),
            BodyError::TypeMismatch { expr } => expr.loc(),
            BodyError::NonBoolCondition { condition } => condition.loc(),
            BodyError::ReturnValueInVoid { return_stmt } => return_stmt.loc(),
            BodyError::VoidReturnInValued { return_stmt } => return_stmt.loc(),
            BodyError::UnreachableStmts { stmts } => {
                ParserLoc::merge_from(stmts.first().unwrap(), stmts.last().unwrap())
            },
            BodyError::InvalidVoidReturningCall { call } => call.loc(),
            BodyError::DuplicateParam { snd } => snd.loc(),
            BodyError::InvalidClassCast { cast } => cast.loc(),
        }
    }
}
