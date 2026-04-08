use std::rc::Rc;

use crate::{
    ByteSource,
    ast::*,
    diagnostics::Diagnostic,
    lexer::{Token, TokenValue},
    parser::ParserLoc,
};

#[derive(Clone, Debug, Default)]
pub enum ParserValue<'src> {
    None,
    Uninitialized,
    #[default]
    Stolen,
    Token(Token),
    Id(Rc<ShipId<'src>>),
    Int(Rc<ShipInt<'src>>),
    Float(Rc<ShipFloat<'src>>),
    String(Rc<ShipString<'src>>),
    Char(Rc<ShipChar<'src>>),
    This(Rc<ShipThis<'src>>),
    Super(Rc<ShipSuper<'src>>),
    Primary(ShipPrimaryAll<'src>),
    ArgsBuilder(Vec<ShipExprAll<'src>>),
    Args(Rc<ShipArgs<'src>>),
    CallExpr(Rc<ShipCallExpr<'src>>),
    MemberAccessExpr(Rc<ShipMemberAccessExpr<'src>>),
    ClassCastExpr(Rc<ShipClassCastExpr<'src>>),
    AssignableExpr(ShipAssignableExprAll<'src>),
    CallableExpr(ShipCallableExprAll<'src>),
    Expr(ShipExprAll<'src>),
    BodyBuilder(Vec<ShipBodyMemberAll<'src>>),
    Body(Rc<ShipBody<'src>>),
    BodyMember(ShipBodyMemberAll<'src>),
    AssignStmt(Rc<ShipAssignStmt<'src>>),
    IfStmt(Rc<ShipIfStmt<'src>>),
    WhileStmt(Rc<ShipWhileStmt<'src>>),
    ReturnStmt(Rc<ShipReturnStmt<'src>>),
    Stmt(ShipStmtAll<'src>),
    VarDef(Rc<ShipVarDef<'src>>),
    MethodBody(ShipMethodBodyAll<'src>),
    Param(Rc<ShipParam<'src>>),
    ParamsBuilder(Vec<Rc<ShipParam<'src>>>),
    Params(Rc<ShipParams<'src>>),
    MethodDef(Rc<ShipMethodDef<'src>>),
    ConsDef(Rc<ShipConsDef<'src>>),
    ClassMember(ShipClassMemberAll<'src>),
    ClassMembersBuilder(Vec<ShipClassMemberAll<'src>>),
    ClassDef(Rc<ShipClassDef<'src>>),
    ClassDefsBuilder(Vec<Rc<ShipClassDef<'src>>>),
    Program(Rc<ShipProgram<'src>>),
}

impl Token {
    pub fn from(value: ParserValue) -> Token {
        match value {
            ParserValue::Token(t) => t,
            other => unreachable!("expected Token, got {:?}", other),
        }
    }
}

impl<'src> ShipId<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Id(n) => n,
            other => unreachable!("expected Id, got {:?}", other),
        }
    }
}

impl<'src> ShipInt<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Int(n) => n,
            other => unreachable!("expected Int, got {:?}", other),
        }
    }
}

impl<'src> ShipFloat<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Float(n) => n,
            other => unreachable!("expected Float, got {:?}", other),
        }
    }
}

impl<'src> ShipString<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::String(n) => n,
            other => unreachable!("expected String, got {:?}", other),
        }
    }
}

impl<'src> ShipChar<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Char(n) => n,
            other => unreachable!("expected Char, got {:?}", other),
        }
    }
}

impl<'src> ShipThis<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::This(n) => n,
            other => unreachable!("expected This, got {:?}", other),
        }
    }
}

impl<'src> ShipSuper<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Super(n) => n,
            other => unreachable!("expected Super, got {:?}", other),
        }
    }
}

impl<'src> ShipPrimaryAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Primary(p) => p,
            other => unreachable!("expected Primary, got {:?}", other),
        }
    }
}

impl<'src> ShipClassCastExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::ClassCastExpr(n) => n,
            other => unreachable!("expected ClassCastExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipMemberAccessExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::MemberAccessExpr(n) => n,
            other => unreachable!("expected MemberAccessExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipArgs<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Args(n) => n,
            other => unreachable!("expected Args, got {:?}", other),
        }
    }
}

impl<'src> ShipCallExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::CallExpr(n) => n,
            other => unreachable!("expected MethodCallExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipAssignableExprAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::AssignableExpr(e) => e,
            other => unreachable!("expected AssignableExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipCallableExprAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::CallableExpr(e) => e,
            other => unreachable!("expected CallableExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipExprAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Expr(e) => e,
            other => unreachable!("expected Expr, got {:?}", other),
        }
    }
}

impl<'src> ShipBodyMemberAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::BodyMember(e) => e,
            other => unreachable!("expected BodyMember, got {:?}", other),
        }
    }
}

impl<'src> ShipBody<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Body(n) => n,
            other => unreachable!("expected Body, got {:?}", other),
        }
    }
}

impl<'src> ShipAssignStmt<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::AssignStmt(n) => n,
            other => unreachable!("expected AssignStmt, got {:?}", other),
        }
    }
}

impl<'src> ShipIfStmt<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::IfStmt(n) => n,
            other => unreachable!("expected IfStmt, got {:?}", other),
        }
    }
}

impl<'src> ShipWhileStmt<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::WhileStmt(n) => n,
            other => unreachable!("expected WhileStmt, got {:?}", other),
        }
    }
}

impl<'src> ShipReturnStmt<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::ReturnStmt(n) => n,
            other => unreachable!("expected ReturnStmt, got {:?}", other),
        }
    }
}

impl<'src> ShipStmtAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Stmt(n) => n,
            other => unreachable!("expected Stmt, got {:?}", other),
        }
    }
}

impl<'src> ShipVarDef<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::VarDef(n) => n,
            other => unreachable!("expected VarDef, got {:?}", other),
        }
    }
}

impl<'src> ShipMethodBodyAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::MethodBody(e) => e,
            other => unreachable!("expected MethodBody, got {:?}", other),
        }
    }
}

impl<'src> ShipParam<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Param(n) => n,
            other => unreachable!("expected Param, got {:?}", other),
        }
    }
}

impl<'src> ShipParams<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Params(n) => n,
            other => unreachable!("expected Params, got {:?}", other),
        }
    }
}

impl<'src> ShipMethodDef<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::MethodDef(n) => n,
            other => unreachable!("expected MethodDef, got {:?}", other),
        }
    }
}

impl<'src> ShipConsDef<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::ConsDef(n) => n,
            other => unreachable!("expected ConsDef, got {:?}", other),
        }
    }
}

impl<'src> ShipClassMemberAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::ClassMember(n) => n,
            other => unreachable!("expected ClassMember, got {:?}", other),
        }
    }
}

impl<'src> ShipClassDef<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::ClassDef(n) => n,
            other => unreachable!("expected ClassDef, got {:?}", other),
        }
    }
}

impl<'src> ShipProgram<'src> {
    pub fn from(value: ParserValue<'src>) -> Rc<Self> {
        match value {
            ParserValue::Program(n) => n,
            other => unreachable!("expected Program, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ArgsBuilder {
    use super::ParserValue;
    use crate::ast::ShipExprAll;

    pub fn from<'src>(value: ParserValue<'src>) -> Vec<ShipExprAll<'src>> {
        match value {
            ParserValue::ArgsBuilder(a) => a,
            other => unreachable!("expected ArgsBuilder, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod BodyBuilder {
    use super::ParserValue;
    use crate::ast::ShipBodyMemberAll;

    pub fn from<'src>(value: ParserValue<'src>) -> Vec<ShipBodyMemberAll<'src>> {
        match value {
            ParserValue::BodyBuilder(a) => a,
            other => unreachable!("expected BodyBuilder, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ParamsBuilder {
    use std::rc::Rc;

    use super::ParserValue;
    use crate::ast::ShipParam;

    pub fn from<'src>(value: ParserValue<'src>) -> Vec<Rc<ShipParam<'src>>> {
        match value {
            ParserValue::ParamsBuilder(a) => a,
            other => unreachable!("expected ParamBuilder, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ClassMembersBuilder {
    use super::ParserValue;
    use crate::ast::ShipClassMemberAll;

    pub fn from<'src>(value: ParserValue<'src>) -> Vec<ShipClassMemberAll<'src>> {
        match value {
            ParserValue::ClassMembersBuilder(a) => a,
            other => unreachable!("expected ClassMemberBuilder, got {:?}", other),
        }
    }
}

#[allow(non_snake_case)]
pub mod ClassDefsBuilder {
    use std::rc::Rc;

    use super::ParserValue;
    use crate::ast::ShipClassDef;

    pub fn from<'src>(value: ParserValue<'src>) -> Vec<Rc<ShipClassDef<'src>>> {
        match value {
            ParserValue::ClassDefsBuilder(a) => a,
            other => unreachable!("expected ClassDefsBuilder, got {:?}", other),
        }
    }
}

impl<'src> ParserValue<'src> {
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

    pub fn new_id(src: &impl ByteSource<'src>, token: Token) -> Self {
        match token.token_value {
            TokenValue::String(_id) => Self::Id(ShipId::new(
                IdData {
                    id: str::from_utf8(src.source(token.loc)).unwrap_or("non utf-8 fragment"),
                },
                token.loc,
                src,
            )),
            other => unreachable!("expected String, got {:?}", other),
        }
    }

    pub fn new_int(src: &impl ByteSource<'src>, token: Token) -> Self {
        match token.token_value {
            TokenValue::Int(int) => Self::Int(ShipInt::new(IntData { int }, token.loc, src)),
            other => unreachable!("expected Integer, got {:?}", other),
        }
    }

    pub fn new_float(src: &impl ByteSource<'src>, token: Token) -> Self {
        match token.token_value {
            TokenValue::Float(float) => {
                Self::Float(ShipFloat::new(FloatData { float }, token.loc, src))
            },
            other => unreachable!("expected Float, got {:?}", other),
        }
    }

    pub fn new_string(src: &impl ByteSource<'src>, token: Token) -> Self {
        match token.token_value {
            TokenValue::String(string) => {
                Self::String(ShipString::new(StringData { string }, token.loc, src))
            },
            other => unreachable!("expected String, got {:?}", other),
        }
    }

    pub fn new_char(src: &impl ByteSource<'src>, token: Token) -> Self {
        match token.token_value {
            TokenValue::Char(char) => Self::Char(ShipChar::new(CharData { char }, token.loc, src)),
            other => unreachable!("expected Char, got {:?}", other),
        }
    }

    pub fn new_this(src: &impl ByteSource<'src>, token: Token) -> Self {
        Self::This(ShipThis::new(ThisData {}, token.loc, src))
    }

    pub fn new_super(src: &impl ByteSource<'src>, token: Token) -> Self {
        Self::Super(ShipSuper::new(SuperData {}, token.loc, src))
    }

    pub fn new_primary(_src: &impl ByteSource<'src>, primary: ShipPrimaryAll<'src>) -> Self {
        Self::Primary(primary)
    }

    pub fn new_class_cast(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        expr: ShipExprAll<'src>,
        class_id: Rc<ShipId<'src>>,
    ) -> Self {
        Self::ClassCastExpr(ShipClassCastExpr::new(ClassCastExprData { expr, class_id }, loc, src))
    }

    pub fn new_member_access(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        expr: ShipExprAll<'src>,
        member_id: Rc<ShipId<'src>>,
    ) -> Self {
        Self::MemberAccessExpr(ShipMemberAccessExpr::new(
            MemberAccessExprData { expr, member_id },
            loc,
            src,
        ))
    }

    pub fn new_args_builder(args: Vec<ShipExprAll<'src>>) -> Self {
        Self::ArgsBuilder(args)
    }

    pub fn new_args(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        exprs: Vec<ShipExprAll<'src>>,
    ) -> Self {
        Self::Args(ShipArgs::new(ArgsData { exprs }, loc, src))
    }

    pub fn new_method_call(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        expr: ShipCallableExprAll<'src>,
        args: Rc<ShipArgs<'src>>,
    ) -> Self {
        Self::CallExpr(ShipCallExpr::new(CallExprData { expr, args }, loc, src))
    }

    pub fn new_assignable_expr(
        _src: &impl ByteSource<'src>,
        expr: ShipAssignableExprAll<'src>,
    ) -> Self {
        Self::AssignableExpr(expr)
    }

    pub fn new_callable_expr(
        _src: &impl ByteSource<'src>,
        expr: ShipCallableExprAll<'src>,
    ) -> Self {
        Self::CallableExpr(expr)
    }

    pub fn new_expr(_src: &impl ByteSource<'src>, expr: ShipExprAll<'src>) -> Self {
        Self::Expr(expr)
    }

    pub fn new_body_builder(args: Vec<ShipBodyMemberAll<'src>>) -> Self {
        Self::BodyBuilder(args)
    }

    pub fn new_body(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        members: Vec<ShipBodyMemberAll<'src>>,
    ) -> Self {
        Self::Body(ShipBody::new(BodyData { members }, loc, src))
    }

    pub fn new_body_member(_src: &impl ByteSource<'src>, member: ShipBodyMemberAll<'src>) -> Self {
        Self::BodyMember(member)
    }

    pub fn new_assign_stmt(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        target: ShipAssignableExprAll<'src>,
        value: ShipExprAll<'src>,
    ) -> Self {
        Self::AssignStmt(ShipAssignStmt::new(AssignStmtData { target, value }, loc, src))
    }

    pub fn new_if_stmt(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        condition: ShipExprAll<'src>,
        then_body: Rc<ShipBody<'src>>,
        else_body: Option<Rc<ShipBody<'src>>>,
    ) -> Self {
        Self::IfStmt(ShipIfStmt::new(IfStmtData { condition, then_body, else_body }, loc, src))
    }

    pub fn new_while_stmt(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        condition: ShipExprAll<'src>,
        body: Rc<ShipBody<'src>>,
    ) -> Self {
        Self::WhileStmt(ShipWhileStmt::new(WhileStmtData { condition, body }, loc, src))
    }

    pub fn new_return_stmt(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        value: Option<ShipExprAll<'src>>,
    ) -> (Self, Rc<ShipReturnStmt<'src>>) {
        let return_stmt = ShipReturnStmt::new(ReturnStmtData { value }, loc, src);
        (Self::ReturnStmt(return_stmt.clone()), return_stmt)
    }

    pub fn new_stmt(_src: &impl ByteSource<'src>, stmt: ShipStmtAll<'src>) -> Self {
        Self::Stmt(stmt)
    }

    pub fn new_var_def(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        var_id: Rc<ShipId<'src>>,
        expr: ShipExprAll<'src>,
    ) -> Self {
        Self::VarDef(ShipVarDef::new(VarDefData { var_id, expr }, loc, src))
    }

    pub fn new_method_body(_src: &impl ByteSource<'src>, body: ShipMethodBodyAll<'src>) -> Self {
        Self::MethodBody(body)
    }

    pub fn new_param(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        name: Rc<ShipId<'src>>,
        var_type: Rc<ShipId<'src>>,
    ) -> Self {
        Self::Param(ShipParam::new(ParamData { name, var_type }, loc, src))
    }

    pub fn new_params_builder(params: Vec<Rc<ShipParam<'src>>>) -> Self {
        Self::ParamsBuilder(params)
    }

    pub fn new_params(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        params: Vec<Rc<ShipParam<'src>>>,
    ) -> Self {
        Self::Params(ShipParams::new(ParamsData { params }, loc, src))
    }

    pub fn new_method_def(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        method_id: Rc<ShipId<'src>>,
        params: Rc<ShipParams<'src>>,
        return_type: Option<Rc<ShipId<'src>>>,
        body: Option<ShipMethodBodyAll<'src>>,
    ) -> Self {
        Self::MethodDef(ShipMethodDef::new(
            MethodDefData { method_id, params, return_type, body },
            loc,
            src,
        ))
    }

    pub fn new_cons_def(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        params: Rc<ShipParams<'src>>,
        body: Rc<ShipBody<'src>>,
    ) -> Self {
        Self::ConsDef(ShipConsDef::new(ConsDefData { params, body }, loc, src))
    }

    pub fn new_class_member(
        _src: &impl ByteSource<'src>,
        member: ShipClassMemberAll<'src>,
    ) -> Self {
        Self::ClassMember(member)
    }

    pub fn new_class_members_builder(members: Vec<ShipClassMemberAll<'src>>) -> Self {
        Self::ClassMembersBuilder(members)
    }

    pub fn new_class_def(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        class_id: Rc<ShipId<'src>>,
        parent_id: Option<Rc<ShipId<'src>>>,
        members: Vec<ShipClassMemberAll<'src>>,
    ) -> Self {
        Self::ClassDef(ShipClassDef::new(ClassDefData { class_id, parent_id, members }, loc, src))
    }

    pub fn new_class_defs_builder(classes: Vec<Rc<ShipClassDef<'src>>>) -> Self {
        Self::ClassDefsBuilder(classes)
    }

    pub fn new_program(
        src: &impl ByteSource<'src>,
        loc: ParserLoc,
        classes: Vec<Rc<ShipClassDef<'src>>>,
    ) -> Self {
        Self::Program(ShipProgram::new(ProgramData { classes }, loc, src))
    }
}
