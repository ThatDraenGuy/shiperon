type ShipId = String;

#[derive(Debug, Clone)]
pub struct Program {
    members: Vec<ShipClassDefinition>,
}

#[derive(Debug, Clone)]
pub struct ShipClassDefinition {
    id: ShipId,
    parent: Option<ShipId>,
    members: Vec<ShipClassMember>,
}

#[derive(Debug, Clone)]
pub enum ShipClassMember {
    VarDef(ShipVarDefinition),
    MethodDef(ShipMethodDefinition),
    ConstructorDef(ShipConstructorDefinition),
}

#[derive(Debug, Clone)]
pub struct ShipMethodDefinition {
    id: ShipId,
    params: Vec<ShipParam>,
    return_type: Option<ShipId>,
    body: Option<ShipMethodBody>,
}

#[derive(Debug, Clone)]
pub struct ShipParam {
    name: ShipId,
    var_type: ShipId,
}

#[derive(Debug, Clone)]
pub enum ShipMethodBody {
    Body(Box<ShipBody>),
    Expr(Box<ShipExpression>),
}

#[derive(Debug, Clone)]
pub struct ShipConstructorDefinition {
    params: Vec<ShipParam>,
    body: Box<ShipBody>,
}

#[derive(Debug, Clone)]
pub struct ShipVarDefinition {
    id: ShipId,
    expr: Box<ShipExpression>,
}

type ShipBody = Vec<ShipBodyMember>;

#[derive(Debug, Clone)]
pub enum ShipBodyMember {
    VarDef(Box<ShipVarDefinition>),
    Stmt(Box<ShipStatement>),
}

#[derive(Debug, Clone)]
pub enum ShipStatement {
    Assign,
    While {
        condition: Box<ShipExpression>,
        body: Box<ShipBody>,
    },
    If {
        condition: Box<ShipExpression>,
        then_body: Box<ShipBody>,
        else_body: Option<Box<ShipBody>>,
    },
    Return {
        value: Option<Box<ShipExpression>>,
    },
}

#[derive(Debug, Clone)]
pub enum ShipExpression {
    ConstructorCall { class_id: ShipId, args: Vec<ShipExpression> },
    MemberAccess { object: Box<ShipExpression>, member_id: ShipId },
    MethodCall { method: Box<ShipExpression>, args: Vec<ShipExpression> },
    Primary(ShipPrimary),
}

#[derive(Debug, Clone)]
pub enum ShipPrimary {
    Int(i32),
    Float(f32),
    This,
}
