use std::{collections::LinkedList, rc::Rc};

use derive_more::Display;

use crate::{
    analyzer::{
        AnalysisError,
        expr::{CallExpr, Expr, ExprModel},
        field::{ClassWithFieldRegistry, FieldModel, WithClassFields},
        registry::{
            ClassId, ClassRegistry, ConsId, ConsRegistry, FieldId, LibClassId, MethodId,
            MethodRegistry, UserClassId, VarId, VarNameRegistryBuilder,
        },
        signature::{
            ClassSignature, MethodSignature, ParamsSignature, WithClassSignature,
            WithMethodSignature,
        },
    },
    ast::*,
    diagnostics::Renderable,
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
    Var(VarId, &'a VarSignature),
    Field(FieldId, &'a FieldModel),
    Global,
}

pub struct ScopeStack<'src> {
    inner: LinkedList<BodyScope<'src>>,
    pub curr_cls: UserClassId,
    pub expected_return: Option<Option<ClassId>>,
}

impl<'src> ScopeStack<'src> {
    fn new_cons(cls: UserClassId) -> Self {
        let inner = LinkedList::new(); //TODO parent fields!!!
        Self { inner, curr_cls: cls, expected_return: None }
    }
    fn new_method(cls: UserClassId, return_type: Option<ClassId>) -> Self {
        let inner = LinkedList::new(); //TODO parent fields!!!
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

    pub fn find_var<'a, V: WithClassSignature<'src> + WithClassFields>(
        &'a self,
        registry: &'a ClassRegistry<V>,
        name: &Rc<ShipId<'src>>,
    ) -> Option<ScopeVar<'a>> {
        for scope in &self.inner {
            if let Some(id) = scope.vars.curr().get_by_name(name.id) {
                return Some(ScopeVar::Var(id, scope.vars.curr().get(&id)));
            }
        }
        if let Ok((field_id, field_model)) = registry.find_field(self.curr_cls.into(), name) {
            return Some(ScopeVar::Field(field_id, field_model));
        }
        None
    }
}

pub struct Body {
    pub stmts: Vec<Stmt>,
    pub return_expr: Option<BodyReturn<ExprModel>>,
}
impl Body {
    fn resolve<'src>(
        registry: &ClassWithFieldRegistry<'src>,
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
                        let init_expr = ExprModel::resolve(registry, scopes, &var_def.expr, errors);
                        let id = scopes.curr_mut().vars.update(var_def.var_id.id, |_maybe_old| {
                            VarSignature { var_type: init_expr.expr_type, mutable: true } //variable shadowing
                        });
                        Stmt::VarDef { id, init_expr }
                    },
                    ShipBodyMemberAll::Stmt(stmt) => match stmt {
                        ShipStmtAll::Assign(assign) => {
                            let value = ExprModel::resolve(registry, scopes, &assign.value, errors);
                            let target = ExprModel::resolve_assignable(
                                registry,
                                &assign.target,
                                scopes,
                                value.expr_type,
                                &assign.value,
                                errors,
                            );
                            Stmt::Assign(target, value)
                        },
                        ShipStmtAll::While(while_node) => {
                            let condition =
                                ExprModel::resolve(registry, scopes, &while_node.condition, errors);
                            if condition.expr_type != LibClassId::Boolean.into() {
                                errors.push(
                                    BodyError::NonBoolCondition {
                                        condition: while_node.condition.clone(),
                                    }
                                    .into(),
                                );
                            }
                            scopes.enter(ScopeType::While);
                            let body = Self::resolve(registry, scopes, &while_node.body, errors);
                            scopes.exit();
                            Stmt::While { condition, body }
                        },
                        ShipStmtAll::If(if_node) => {
                            let condition =
                                ExprModel::resolve(registry, scopes, &if_node.condition, errors);
                            if condition.expr_type != LibClassId::Boolean.into() {
                                errors.push(
                                    BodyError::NonBoolCondition {
                                        condition: if_node.condition.clone(),
                                    }
                                    .into(),
                                );
                            }
                            scopes.enter(ScopeType::If);
                            let then_body =
                                Self::resolve(registry, scopes, &if_node.then_body, errors);
                            scopes.exit();
                            let else_body = if_node.else_body.as_ref().map(|else_body_node| {
                                scopes.enter(ScopeType::If);
                                let res = Self::resolve(registry, scopes, else_body_node, errors);
                                scopes.exit();
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
                                ExprModel::resolve_callable(registry, call_node, scopes, errors);
                            Stmt::Call(call_expr.1)
                        },
                        ShipStmtAll::Return(return_stmt) => {
                            return Self {
                                stmts,
                                return_expr: match scopes.expected_return {
                                    // parser ensures no statements after return in bodies
                                    Some(return_type) => {
                                        Some(match (return_type, &return_stmt.value) {
                                            (None, None) => BodyReturn::Void,
                                            (Some(expected_type), Some(return_expr)) => {
                                                let expr = ExprModel::resolve(
                                                    registry,
                                                    scopes,
                                                    return_expr,
                                                    errors,
                                                );
                                                if registry
                                                    .registry()
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
            Self { stmts, return_expr: Some(BodyReturn::Never) }
        } else {
            Self { stmts, return_expr: None }
        }
    }
}

pub enum AssignTarget {
    Var(VarId),
    Field(ExprModel, FieldId),
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
        registry: &ClassWithFieldRegistry<'src>,
        cls_id: UserClassId,
        cons_id: ConsId,
        def: &Rc<ShipConsDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let signature = &registry.get(&cls_id).class_signature().constructors.get(&cons_id).1;
        let mut scopes = ScopeStack::new_cons(cls_id);
        let body = Body::resolve(registry, &mut scopes, &def.body, errors);
        Self { body }
    }
}

pub type ConsBodyRegistry = ConsRegistry<ConsBody>;

pub struct MethodBody {
    pub body: Body,
}

impl MethodBody {
    pub fn resolve<'src>(
        registry: &ClassWithFieldRegistry<'src>,
        cls_id: UserClassId,
        method_id: MethodId,
        def: &Rc<ShipMethodDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let signature = registry
            .get(&cls_id)
            .class_signature()
            .methods
            .registry()
            .get_method(&method_id)
            .method_signature();

        let mut scopes = ScopeStack::new_method(cls_id, signature.return_type);
        let body = match &def.body {
            Some(ShipMethodBodyAll::Expr(expr)) => {
                let expr_model = ExprModel::resolve(registry, &scopes, expr, errors);
                Body { stmts: vec![], return_expr: Some(BodyReturn::Value(expr_model)) }
            },
            Some(ShipMethodBodyAll::Body(body)) => {
                Body::resolve(registry, &mut scopes, body, errors)
            },
            None => todo!(),
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
}
impl<'src> Renderable<'src> for BodyError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
