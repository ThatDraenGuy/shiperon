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
    Id(ShipId),
    Primary(ShipPrimary),
    Expr(ShipExpression),
    Stmt(ShipStatement),
    Args(Vec<ShipExpression>),
    Params(Vec<ShipParam>),
    Param(ShipParam),
    Body(ShipBody),
    BodyMember(ShipBodyMember),
    VarDef(ShipVarDefinition),
    MethodBody(ShipMethodBody),
    MethodDef(ShipMethodDefinition),
    ConstructorDef(ShipConstructorDefinition),
    ClassMember(ShipClassMember),
    ClassMembers(Vec<ShipClassMember>),
    ClassDef(ShipClassDefinition),
    ClassDefs(Vec<ShipClassDefinition>),
    Program(ShipProgram),
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

impl ShipStatement {
    pub fn from(value: ParserValue) -> ShipStatement {
        match value {
            ParserValue::Stmt(s) => s,
            other => unreachable!("expected Stmt, got {:?}", other),
        }
    }
}

impl ShipBody {
    pub fn from(value: ParserValue) -> ShipBody {
        match value {
            ParserValue::Body(b) => b,
            other => unreachable!("expected Body, got {:?}", other),
        }
    }
}

impl ShipBodyMember {
    pub fn from(value: ParserValue) -> ShipBodyMember {
        match value {
            ParserValue::BodyMember(m) => m,
            other => unreachable!("expected BodyMember, got {:?}", other),
        }
    }
}

impl ShipVarDefinition {
    pub fn from(value: ParserValue) -> ShipVarDefinition {
        match value {
            ParserValue::VarDef(d) => d,
            other => unreachable!("expected VarDef, got {:?}", other),
        }
    }
}

impl ShipMethodBody {
    pub fn from(value: ParserValue) -> ShipMethodBody {
        match value {
            ParserValue::MethodBody(b) => b,
            other => unreachable!("expected MethodBody, got {:?}", other),
        }
    }
}

impl ShipParam {
    pub fn from(value: ParserValue) -> ShipParam {
        match value {
            ParserValue::Param(p) => p,
            other => unreachable!("expected Param, got {:?}", other),
        }
    }
}

impl ShipMethodDefinition {
    pub fn from(value: ParserValue) -> ShipMethodDefinition {
        match value {
            ParserValue::MethodDef(d) => d,
            other => unreachable!("expected MethodDef, got {:?}", other),
        }
    }
}

impl ShipConstructorDefinition {
    pub fn from(value: ParserValue) -> ShipConstructorDefinition {
        match value {
            ParserValue::ConstructorDef(d) => d,
            other => unreachable!("expected ConstructorDef, got {:?}", other),
        }
    }
}

impl ShipClassMember {
    pub fn from(value: ParserValue) -> ShipClassMember {
        match value {
            ParserValue::ClassMember(m) => m,
            other => unreachable!("expected ClassMember, got {:?}", other),
        }
    }
}

impl ShipClassDefinition {
    pub fn from(value: ParserValue) -> ShipClassDefinition {
        match value {
            ParserValue::ClassDef(d) => d,
            other => unreachable!("expected ClassDef, got {:?}", other),
        }
    }
}

impl ShipProgram {
    pub fn from(value: ParserValue) -> ShipProgram {
        match value {
            ParserValue::Program(p) => p,
            other => unreachable!("expected Program, got {:?}", other),
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

#[allow(non_snake_case)]
pub mod Params {
    use super::ParserValue;
    use crate::ast::ShipParam;

    pub fn from(value: ParserValue) -> Vec<ShipParam> {
        match value {
            ParserValue::Params(p) => p,
            other => unreachable!("expected Params, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ClassMembers {
    use super::ParserValue;
    use crate::ast::ShipClassMember;

    pub fn from(value: ParserValue) -> Vec<ShipClassMember> {
        match value {
            ParserValue::ClassMembers(m) => m,
            other => unreachable!("expected ClassMembers, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ClassDefs {
    use super::ParserValue;
    use crate::ast::ShipClassDefinition;

    pub fn from(value: ParserValue) -> Vec<ShipClassDefinition> {
        match value {
            ParserValue::ClassDefs(d) => d,
            other => unreachable!("expected ClassDefs, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod Id {
    use super::ParserValue;
    use crate::ast::ShipId;

    pub fn from(value: ParserValue) -> ShipId {
        match value {
            ParserValue::Id(i) => i,
            other => unreachable!("expected Id, got {:?}", other),
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

    pub fn new_id(token: Token) -> Self {
        match token.token_value {
            TokenValue::String(s) => Self::Id(s),
            other => unreachable!("expected String, got {:?}", other),
        }
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

    pub fn new_primary_id(id: ShipId) -> Self {
        Self::Primary(ShipPrimary::Id(id))
    }

    pub fn new_expr_constructor_call(class_id: ShipId, args: Vec<ShipExpression>) -> Self {
        Self::Expr(ShipExpression::ConstructorCall { class_id, args })
    }

    pub fn new_expr_member_access(object: ShipExpression, member_id: ShipId) -> Self {
        Self::Expr(ShipExpression::MemberAccess { expr: Box::new(object), member_id })
    }

    pub fn new_expr_method_call(method: ShipExpression, args: Vec<ShipExpression>) -> Self {
        Self::Expr(ShipExpression::MethodCall { expr: Box::new(method), args })
    }

    pub fn new_expr_primary(primary: ShipPrimary) -> Self {
        Self::Expr(ShipExpression::Primary(primary))
    }

    pub fn new_stmt_assign(target: ShipExpression, value: ShipExpression) -> Self {
        Self::Stmt(ShipStatement::Assign { target: Box::new(target), value: Box::new(value) })
    }

    pub fn new_stmt_while(condition: ShipExpression, body: ShipBody) -> Self {
        Self::Stmt(ShipStatement::While { condition: Box::new(condition), body: Box::new(body) })
    }

    pub fn new_stmt_if(
        condition: ShipExpression,
        then_body: ShipBody,
        else_body: Option<ShipBody>,
    ) -> Self {
        Self::Stmt(ShipStatement::If {
            condition: Box::new(condition),
            then_body: Box::new(then_body),
            else_body: else_body.map(Box::new),
        })
    }

    pub fn new_stmt_return(value: Option<ShipExpression>) -> Self {
        Self::Stmt(ShipStatement::Return { value: value.map(Box::new) })
    }

    pub fn new_args(args: Vec<ShipExpression>) -> Self {
        Self::Args(args)
    }

    pub fn new_body(members: ShipBody) -> Self {
        Self::Body(members)
    }

    pub fn new_body_member_stmt(stmt: ShipStatement) -> Self {
        Self::BodyMember(ShipBodyMember::Stmt(Box::new(stmt)))
    }

    pub fn new_body_member_var_def(var_def: ShipVarDefinition) -> Self {
        Self::BodyMember(ShipBodyMember::VarDef(Box::new(var_def)))
    }

    pub fn new_var_def(var_id: ShipId, expr: ShipExpression) -> Self {
        Self::VarDef(ShipVarDefinition { var_id, expr: Box::new(expr) })
    }

    pub fn new_method_body(body: ShipBody) -> Self {
        Self::MethodBody(ShipMethodBody::Body(Box::new(body)))
    }

    pub fn new_method_body_short(expr: ShipExpression) -> Self {
        Self::MethodBody(ShipMethodBody::Expr(Box::new(expr)))
    }

    pub fn new_param(param_id: ShipId, type_id: ShipId) -> Self {
        Self::Param(ShipParam { name: param_id, var_type: type_id })
    }

    pub fn new_params(params: Vec<ShipParam>) -> Self {
        Self::Params(params)
    }

    pub fn new_method_def(
        method_id: ShipId,
        params: Vec<ShipParam>,
        return_type: Option<ShipId>,
        body: Option<ShipMethodBody>,
    ) -> Self {
        Self::MethodDef(ShipMethodDefinition {
            method_id,
            params,
            return_type,
            body: body.map(Box::new),
        })
    }

    pub fn new_constructor_def(params: Vec<ShipParam>, body: ShipBody) -> Self {
        Self::ConstructorDef(ShipConstructorDefinition { params, body: Box::new(body) })
    }

    pub fn new_class_member_var_def(var_def: ShipVarDefinition) -> Self {
        Self::ClassMember(ShipClassMember::VarDef(var_def))
    }

    pub fn new_class_member_method_def(method_def: ShipMethodDefinition) -> Self {
        Self::ClassMember(ShipClassMember::MethodDef(method_def))
    }

    pub fn new_class_member_constructor_def(constructor_def: ShipConstructorDefinition) -> Self {
        Self::ClassMember(ShipClassMember::ConstructorDef(constructor_def))
    }

    pub fn new_class_members(members: Vec<ShipClassMember>) -> Self {
        Self::ClassMembers(members)
    }

    pub fn new_class_def(
        class_id: ShipId,
        parent_id: Option<ShipId>,
        members: Vec<ShipClassMember>,
    ) -> Self {
        Self::ClassDef(ShipClassDefinition { class_id, parent_id, members })
    }

    pub fn new_class_defs(defs: Vec<ShipClassDefinition>) -> Self {
        Self::ClassDefs(defs)
    }

    pub fn new_program(classes: Vec<ShipClassDefinition>) -> Self {
        Self::Program(ShipProgram { classes })
    }
}
