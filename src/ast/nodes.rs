use std::{fmt::Display, rc::Rc};

use serde::Serialize;

use crate::parser::WithParserLoc;

use super::{Node, NodeData};

#[derive(Debug, Clone, Serialize)]
pub struct ProgramData<'src> {
    pub classes: Vec<Rc<ShipClassDef<'src>>>,
}
impl<'src> NodeData for ProgramData<'src> {
    fn name() -> &'static str {
        "Program"
    }
}
pub type ShipProgram<'src> = Node<'src, ProgramData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct ClassDefData<'src> {
    pub class_id: Rc<ShipId<'src>>,
    pub parent_id: Option<Rc<ShipId<'src>>>,
    pub members: Vec<ShipClassMemberAll<'src>>,
}
impl<'src> NodeData for ClassDefData<'src> {
    fn name() -> &'static str {
        "ClassDef"
    }
}
pub type ShipClassDef<'src> = Node<'src, ClassDefData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub enum ShipClassMemberAll<'src> {
    VarDef(Rc<ShipVarDef<'src>>),
    MethodDef(Rc<ShipMethodDef<'src>>),
    ConsDef(Rc<ShipConsDef<'src>>),
}

impl<'src> Display for ShipClassMemberAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipClassMemberAll::VarDef(node) => node.fmt(f),
            ShipClassMemberAll::MethodDef(node) => node.fmt(f),
            ShipClassMemberAll::ConsDef(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodDefData<'src> {
    pub method_id: Rc<ShipId<'src>>,
    pub params: Rc<ShipParams<'src>>,
    pub return_type: Option<Rc<ShipId<'src>>>,
    pub body: Option<ShipMethodBodyAll<'src>>,
}
impl<'src> NodeData for MethodDefData<'src> {
    fn name() -> &'static str {
        "MethodDef"
    }
}
pub type ShipMethodDef<'src> = Node<'src, MethodDefData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct ParamData<'src> {
    pub name: Rc<ShipId<'src>>,
    pub var_type: Rc<ShipId<'src>>,
}
impl<'src> NodeData for ParamData<'src> {
    fn name() -> &'static str {
        "Param"
    }
}
pub type ShipParam<'src> = Node<'src, ParamData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct ParamsData<'src> {
    pub params: Vec<Rc<ShipParam<'src>>>,
}
impl<'src> NodeData for ParamsData<'src> {
    fn name() -> &'static str {
        "Params"
    }
}
pub type ShipParams<'src> = Node<'src, ParamsData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub enum ShipMethodBodyAll<'src> {
    Body(Rc<ShipBody<'src>>),
    Expr(ShipExprAll<'src>),
}
impl<'src> WithParserLoc for ShipMethodBodyAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipMethodBodyAll::Body(node) => node.loc(),
            ShipMethodBodyAll::Expr(ship_expr_all) => ship_expr_all.loc(),
        }
    }
}

impl<'src> Display for ShipMethodBodyAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipMethodBodyAll::Body(node) => node.fmt(f),
            ShipMethodBodyAll::Expr(expr) => expr.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsDefData<'src> {
    pub params: Rc<ShipParams<'src>>,
    pub body: Rc<ShipBody<'src>>,
}
impl<'src> NodeData for ConsDefData<'src> {
    fn name() -> &'static str {
        "ConsDef"
    }
}
pub type ShipConsDef<'src> = Node<'src, ConsDefData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct VarDefData<'src> {
    pub var_id: Rc<ShipId<'src>>,
    pub expr: ShipExprAll<'src>,
}
impl<'src> NodeData for VarDefData<'src> {
    fn name() -> &'static str {
        "VarDef"
    }
}
pub type ShipVarDef<'src> = Node<'src, VarDefData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct BodyData<'src> {
    pub members: Vec<ShipBodyMemberAll<'src>>,
}
impl<'src> NodeData for BodyData<'src> {
    fn name() -> &'static str {
        "Body"
    }
}
pub type ShipBody<'src> = Node<'src, BodyData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub enum ShipBodyMemberAll<'src> {
    VarDef(Rc<ShipVarDef<'src>>),
    Stmt(ShipStmtAll<'src>),
}

impl<'src> Display for ShipBodyMemberAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipBodyMemberAll::VarDef(node) => node.fmt(f),
            ShipBodyMemberAll::Stmt(stmt) => stmt.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipStmtAll<'src> {
    Assign(Rc<ShipAssignStmt<'src>>),
    While(Rc<ShipWhileStmt<'src>>),
    If(Rc<ShipIfStmt<'src>>),
    Return(Rc<ShipReturnStmt<'src>>),
    Call(Rc<ShipCallExpr<'src>>),
}

impl<'src> Display for ShipStmtAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipStmtAll::Assign(node) => node.fmt(f),
            ShipStmtAll::While(node) => node.fmt(f),
            ShipStmtAll::If(node) => node.fmt(f),
            ShipStmtAll::Return(node) => node.fmt(f),
            ShipStmtAll::Call(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignStmtData<'src> {
    pub target: ShipAssignableExprAll<'src>,
    pub value: ShipExprAll<'src>,
}
impl<'src> NodeData for AssignStmtData<'src> {
    fn name() -> &'static str {
        "AssignStmt"
    }
}
pub type ShipAssignStmt<'src> = Node<'src, AssignStmtData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct WhileStmtData<'src> {
    pub condition: ShipExprAll<'src>,
    pub body: Rc<ShipBody<'src>>,
}
impl<'src> NodeData for WhileStmtData<'src> {
    fn name() -> &'static str {
        "WhileStmt"
    }
}
pub type ShipWhileStmt<'src> = Node<'src, WhileStmtData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct IfStmtData<'src> {
    pub condition: ShipExprAll<'src>,
    pub then_body: Rc<ShipBody<'src>>,
    pub else_body: Option<Rc<ShipBody<'src>>>,
}
impl<'src> NodeData for IfStmtData<'src> {
    fn name() -> &'static str {
        "IfStmt"
    }
}
pub type ShipIfStmt<'src> = Node<'src, IfStmtData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct ReturnStmtData<'src> {
    pub value: Option<ShipExprAll<'src>>,
}
impl<'src> NodeData for ReturnStmtData<'src> {
    fn name() -> &'static str {
        "ReturnStmt"
    }
}
pub type ShipReturnStmt<'src> = Node<'src, ReturnStmtData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub enum ShipExprAll<'src> {
    MemberAccess(Rc<ShipMemberAccessExpr<'src>>),
    Call(Rc<ShipCallExpr<'src>>),
    Primary(ShipPrimaryAll<'src>),
    ClassCast(Rc<ShipClassCastExpr<'src>>),
}
impl<'src> WithParserLoc for ShipExprAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipExprAll::MemberAccess(node) => node.loc(),
            ShipExprAll::Call(node) => node.loc(),
            ShipExprAll::Primary(primary) => primary.loc(),
            ShipExprAll::ClassCast(node) => node.loc(),
        }
    }
}

impl<'src> Display for ShipExprAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipExprAll::MemberAccess(node) => node.fmt(f),
            ShipExprAll::Call(node) => node.fmt(f),
            ShipExprAll::Primary(primary) => primary.fmt(f),
            ShipExprAll::ClassCast(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipAssignableExprAll<'src> {
    MemberAccess(Rc<ShipMemberAccessExpr<'src>>),
    Variable(Rc<ShipId<'src>>),
}
impl<'src> WithParserLoc for ShipAssignableExprAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipAssignableExprAll::MemberAccess(node) => node.loc(),
            ShipAssignableExprAll::Variable(node) => node.loc(),
        }
    }
}
impl<'src> Display for ShipAssignableExprAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipAssignableExprAll::MemberAccess(node) => node.fmt(f),
            ShipAssignableExprAll::Variable(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipCallableExprAll<'src> {
    MemberAccess(Rc<ShipMemberAccessExpr<'src>>),
    This(Rc<ShipThis<'src>>),
    Super(Rc<ShipSuper<'src>>),
    Cons(Rc<ShipId<'src>>),
}
impl<'src> WithParserLoc for ShipCallableExprAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipCallableExprAll::MemberAccess(node) => node.loc(),
            ShipCallableExprAll::This(node) => node.loc(),
            ShipCallableExprAll::Super(node) => node.loc(),
            ShipCallableExprAll::Cons(node) => node.loc(),
        }
    }
}
impl<'src> Display for ShipCallableExprAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipCallableExprAll::MemberAccess(node) => node.fmt(f),
            ShipCallableExprAll::This(node) => node.fmt(f),
            ShipCallableExprAll::Super(node) => node.fmt(f),
            ShipCallableExprAll::Cons(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassCastExprData<'src> {
    pub expr: ShipExprAll<'src>,
    pub class_id: Rc<ShipId<'src>>,
}
impl<'src> NodeData for ClassCastExprData<'src> {
    fn name() -> &'static str {
        "ClassCast"
    }
}
pub type ShipClassCastExpr<'src> = Node<'src, ClassCastExprData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct MemberAccessExprData<'src> {
    pub expr: ShipExprAll<'src>,
    pub member_id: Rc<ShipId<'src>>,
}
impl<'src> NodeData for MemberAccessExprData<'src> {
    fn name() -> &'static str {
        "MemberAccess"
    }
}
pub type ShipMemberAccessExpr<'src> = Node<'src, MemberAccessExprData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct CallExprData<'src> {
    pub expr: ShipCallableExprAll<'src>,
    pub args: Rc<ShipArgs<'src>>,
}
impl<'src> NodeData for CallExprData<'src> {
    fn name() -> &'static str {
        "MethodCall"
    }
}
pub type ShipCallExpr<'src> = Node<'src, CallExprData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub struct ArgsData<'src> {
    pub exprs: Vec<ShipExprAll<'src>>,
}
impl<'src> NodeData for ArgsData<'src> {
    fn name() -> &'static str {
        "Args"
    }
}
pub type ShipArgs<'src> = Node<'src, ArgsData<'src>>;

#[derive(Debug, Clone, Serialize)]
pub enum ShipPrimaryAll<'src> {
    Int(Rc<ShipInt<'src>>),
    Float(Rc<ShipFloat<'src>>),
    String(Rc<ShipString<'src>>),
    Char(Rc<ShipChar<'src>>),
    This(Rc<ShipThis<'src>>),
    Id(Rc<ShipId<'src>>),
}
impl<'src> WithParserLoc for ShipPrimaryAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipPrimaryAll::Int(node) => node.loc(),
            ShipPrimaryAll::Float(node) => node.loc(),
            ShipPrimaryAll::String(node) => node.loc(),
            ShipPrimaryAll::Char(node) => node.loc(),
            ShipPrimaryAll::This(node) => node.loc(),
            ShipPrimaryAll::Id(node) => node.loc(),
        }
    }
}

impl<'src> Display for ShipPrimaryAll<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipPrimaryAll::Int(node) => node.fmt(f),
            ShipPrimaryAll::Float(node) => node.fmt(f),
            ShipPrimaryAll::String(node) => node.fmt(f),
            ShipPrimaryAll::Char(node) => node.fmt(f),
            ShipPrimaryAll::This(node) => node.fmt(f),
            ShipPrimaryAll::Id(node) => node.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IntData {
    pub int: i32,
}
impl NodeData for IntData {
    fn name() -> &'static str {
        "Int"
    }
}
pub type ShipInt<'src> = Node<'src, IntData>;

#[derive(Debug, Clone, Serialize)]
pub struct FloatData {
    pub float: f32,
}
impl NodeData for FloatData {
    fn name() -> &'static str {
        "Float"
    }
}
pub type ShipFloat<'src> = Node<'src, FloatData>;

#[derive(Debug, Clone, Serialize)]
pub struct StringData {
    pub string: String,
}
impl NodeData for StringData {
    fn name() -> &'static str {
        "String"
    }
}
pub type ShipString<'src> = Node<'src, StringData>;

#[derive(Debug, Clone, Serialize)]
pub struct CharData {
    pub char: char,
}
impl NodeData for CharData {
    fn name() -> &'static str {
        "Char"
    }
}
pub type ShipChar<'src> = Node<'src, CharData>;

#[derive(Debug, Clone, Serialize)]
pub struct ThisData {}
impl NodeData for ThisData {
    fn name() -> &'static str {
        "This"
    }
}
pub type ShipThis<'src> = Node<'src, ThisData>;

#[derive(Debug, Clone, Serialize)]
pub struct SuperData {}
impl NodeData for SuperData {
    fn name() -> &'static str {
        "Super"
    }
}
pub type ShipSuper<'src> = Node<'src, SuperData>;

#[derive(Debug, Clone, Serialize)]
pub struct IdData<'src> {
    pub id: &'src str,
}
impl<'src> NodeData for IdData<'src> {
    fn name() -> &'static str {
        "Id"
    }
}
pub type ShipId<'src> = Node<'src, IdData<'src>>;
