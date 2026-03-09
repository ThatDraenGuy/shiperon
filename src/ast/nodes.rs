use std::rc::Rc;

use crate::parser::WithParserLoc;

use super::{Node, NodeData};

#[derive(Debug, Clone)]
pub struct ProgramData<'src> {
    pub classes: Vec<Rc<ShipClassDef<'src>>>,
}
impl<'src> NodeData for ProgramData<'src> {}
pub type ShipProgram<'src> = Node<'src, ProgramData<'src>>;

#[derive(Debug, Clone)]
pub struct ClassDefData<'src> {
    pub class_id: Rc<ShipId<'src>>,
    pub parent_id: Option<Rc<ShipId<'src>>>,
    pub members: Vec<Rc<ShipClassMemberAll<'src>>>,
}
impl<'src> NodeData for ClassDefData<'src> {}
pub type ShipClassDef<'src> = Node<'src, ClassDefData<'src>>;

#[derive(Debug, Clone)]
pub enum ShipClassMemberAll<'src> {
    VarDef(ShipVarDef<'src>),
    MethodDef(ShipMethodDef<'src>),
    ConsDef(ShipConsDef<'src>),
}

#[derive(Debug, Clone)]
pub struct MethodDefData<'src> {
    pub method_id: Rc<ShipId<'src>>,
    pub params: Vec<Rc<ShipParam<'src>>>,
    pub return_type: Option<Rc<ShipId<'src>>>,
    pub body: Option<Rc<ShipMethodBodyAll<'src>>>,
}
impl<'src> NodeData for MethodDefData<'src> {}
pub type ShipMethodDef<'src> = Node<'src, MethodDefData<'src>>;

#[derive(Debug, Clone)]
pub struct ParamData<'src> {
    pub name: Rc<ShipId<'src>>,
    pub var_type: Rc<ShipId<'src>>,
}
impl<'src> NodeData for ParamData<'src> {}
pub type ShipParam<'src> = Node<'src, ParamData<'src>>;

#[derive(Debug, Clone)]
pub enum ShipMethodBodyAll<'src> {
    Body(ShipBody<'src>),
    Expr(ShipExpressionAll<'src>),
}

#[derive(Debug, Clone)]
pub struct ConsDefData<'src> {
    pub params: Vec<Rc<ShipParam<'src>>>,
    pub body: Rc<ShipBody<'src>>,
}
impl<'src> NodeData for ConsDefData<'src> {}
pub type ShipConsDef<'src> = Node<'src, ConsDefData<'src>>;

#[derive(Debug, Clone)]
pub struct VarDefData<'src> {
    pub var_id: Rc<ShipId<'src>>,
    pub expr: Rc<ShipExpressionAll<'src>>,
}
impl<'src> NodeData for VarDefData<'src> {}
pub type ShipVarDef<'src> = Node<'src, VarDefData<'src>>;

#[derive(Debug, Clone)]
pub struct BodyData<'src> {
    pub members: Vec<Rc<ShipBodyMemberAll<'src>>>,
}
impl<'src> NodeData for BodyData<'src> {}
pub type ShipBody<'src> = Node<'src, BodyData<'src>>;

#[derive(Debug, Clone)]
pub enum ShipBodyMemberAll<'src> {
    VarDef(ShipVarDef<'src>),
    Stmt(ShipStatementAll<'src>),
}

#[derive(Debug, Clone)]
pub enum ShipStatementAll<'src> {
    Assign(ShipAssignStmt<'src>),
    While(ShipWhileStmt<'src>),
    If(ShipIfStmt<'src>),
    Return(ShipReturnStmt<'src>),
    MethodCall(ShipMethodCallExpr<'src>),
    ConsCall(ShipConsCallExpr<'src>),
}

#[derive(Debug, Clone)]
pub struct AssignStmtData<'src> {
    pub target: Rc<ShipExpressionAll<'src>>,
    pub value: Rc<ShipExpressionAll<'src>>,
}
impl<'src> NodeData for AssignStmtData<'src> {}
pub type ShipAssignStmt<'src> = Node<'src, AssignStmtData<'src>>;

#[derive(Debug, Clone)]
pub struct WhileStmtData<'src> {
    pub condition: Rc<ShipExpressionAll<'src>>,
    pub body: Rc<ShipBody<'src>>,
}
impl<'src> NodeData for WhileStmtData<'src> {}
pub type ShipWhileStmt<'src> = Node<'src, WhileStmtData<'src>>;

#[derive(Debug, Clone)]
pub struct IfStmtData<'src> {
    pub condition: Rc<ShipExpressionAll<'src>>,
    pub then_body: Rc<ShipBody<'src>>,
    pub else_body: Option<Rc<ShipBody<'src>>>,
}
impl<'src> NodeData for IfStmtData<'src> {}
pub type ShipIfStmt<'src> = Node<'src, IfStmtData<'src>>;

#[derive(Debug, Clone)]
pub struct ReturnStmtData<'src> {
    pub value: Option<Rc<ShipExpressionAll<'src>>>,
}
impl<'src> NodeData for ReturnStmtData<'src> {}
pub type ShipReturnStmt<'src> = Node<'src, ReturnStmtData<'src>>;

#[derive(Debug, Clone)]
pub enum ShipExpressionAll<'src> {
    ConsCall(ShipConsCallExpr<'src>),
    MemberAccess(ShipMemberAccessExpr<'src>),
    MethodCall(ShipMethodCallExpr<'src>),
    Primary(ShipPrimaryAll<'src>),
}
impl<'src> WithParserLoc for ShipExpressionAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipExpressionAll::ConsCall(node) => node.loc(),
            ShipExpressionAll::MemberAccess(node) => node.loc(),
            ShipExpressionAll::MethodCall(node) => node.loc(),
            ShipExpressionAll::Primary(primary) => primary.loc(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsCallExprData<'src> {
    pub class_id: Rc<ShipId<'src>>,
    pub args: Rc<ShipArgs<'src>>,
}
impl<'src> NodeData for ConsCallExprData<'src> {}
pub type ShipConsCallExpr<'src> = Node<'src, ConsCallExprData<'src>>;

#[derive(Debug, Clone)]
pub struct MemberAccessExprData<'src> {
    pub expr: Rc<ShipExpressionAll<'src>>,
    pub member_id: Rc<ShipId<'src>>,
}
impl<'src> NodeData for MemberAccessExprData<'src> {}
pub type ShipMemberAccessExpr<'src> = Node<'src, MemberAccessExprData<'src>>;

#[derive(Debug, Clone)]
pub struct MethodCallExprData<'src> {
    pub expr: Rc<ShipExpressionAll<'src>>,
    pub args: Rc<ShipArgs<'src>>,
}
impl<'src> NodeData for MethodCallExprData<'src> {}
pub type ShipMethodCallExpr<'src> = Node<'src, MethodCallExprData<'src>>;

#[derive(Debug, Clone)]
pub struct ArgsData<'src> {
    pub exprs: Vec<Rc<ShipExpressionAll<'src>>>,
}
impl<'src> NodeData for ArgsData<'src> {}
pub type ShipArgs<'src> = Node<'src, ArgsData<'src>>;

#[derive(Debug, Clone)]
pub enum ShipPrimaryAll<'src> {
    Int(ShipInt<'src>),
    Float(ShipFloat<'src>),
    This(ShipThis<'src>),
    Id(ShipId<'src>),
}
impl<'src> WithParserLoc for ShipPrimaryAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipPrimaryAll::Int(node) => node.loc(),
            ShipPrimaryAll::Float(node) => node.loc(),
            ShipPrimaryAll::This(node) => node.loc(),
            ShipPrimaryAll::Id(node) => node.loc(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntData {
    pub int: i32,
}
impl NodeData for IntData {}
pub type ShipInt<'src> = Node<'src, IntData>;

#[derive(Debug, Clone)]
pub struct FloatData {
    pub float: f32,
}
impl NodeData for FloatData {}
pub type ShipFloat<'src> = Node<'src, FloatData>;

#[derive(Debug, Clone)]
pub struct ThisData {}
impl NodeData for ThisData {}
pub type ShipThis<'src> = Node<'src, ThisData>;

#[derive(Debug, Clone)]
pub struct IdData {
    pub id: String,
}
impl NodeData for IdData {}
pub type ShipId<'src> = Node<'src, IdData>;
