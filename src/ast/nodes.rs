use std::rc::Rc;

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

#[derive(Debug, Clone, Serialize)]
pub enum ShipStmtAll<'src> {
    Assign(Rc<ShipAssignStmt<'src>>),
    While(Rc<ShipWhileStmt<'src>>),
    If(Rc<ShipIfStmt<'src>>),
    Return(Rc<ShipReturnStmt<'src>>),
    MethodCall(Rc<ShipMethodCallExpr<'src>>),
    ConsCall(Rc<ShipConsCallExpr<'src>>),
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignStmtData<'src> {
    pub target: ShipExprAll<'src>,
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
    ConsCall(Rc<ShipConsCallExpr<'src>>),
    MemberAccess(Rc<ShipMemberAccessExpr<'src>>),
    MethodCall(Rc<ShipMethodCallExpr<'src>>),
    Primary(ShipPrimaryAll<'src>),
}
impl<'src> WithParserLoc for ShipExprAll<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            ShipExprAll::ConsCall(node) => node.loc(),
            ShipExprAll::MemberAccess(node) => node.loc(),
            ShipExprAll::MethodCall(node) => node.loc(),
            ShipExprAll::Primary(primary) => primary.loc(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsCallExprData<'src> {
    pub class_id: Rc<ShipId<'src>>,
    pub args: Rc<ShipArgs<'src>>,
}
impl<'src> NodeData for ConsCallExprData<'src> {
    fn name() -> &'static str {
        "ConsCall"
    }
}
pub type ShipConsCallExpr<'src> = Node<'src, ConsCallExprData<'src>>;

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
pub struct MethodCallExprData<'src> {
    pub expr: ShipExprAll<'src>,
    pub args: Rc<ShipArgs<'src>>,
}
impl<'src> NodeData for MethodCallExprData<'src> {
    fn name() -> &'static str {
        "MethodCall"
    }
}
pub type ShipMethodCallExpr<'src> = Node<'src, MethodCallExprData<'src>>;

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
    This(Rc<ShipThis<'src>>),
    Id(Rc<ShipId<'src>>),
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
pub struct ThisData {}
impl NodeData for ThisData {
    fn name() -> &'static str {
        "This"
    }
}
pub type ShipThis<'src> = Node<'src, ThisData>;

#[derive(Debug, Clone, Serialize)]
pub struct IdData {
    pub id: String,
}
impl NodeData for IdData {
    fn name() -> &'static str {
        "Id"
    }
}
pub type ShipId<'src> = Node<'src, IdData>;
