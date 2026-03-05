use serde::Serialize;

pub type ShipId = String;

#[derive(Debug, Clone, Serialize)]
pub struct ShipProgram {
    pub classes: Vec<ShipClassDefinition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipClassDefinition {
    pub class_id: ShipId,
    pub parent_id: Option<ShipId>,
    pub members: Vec<ShipClassMember>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipClassMember {
    VarDef(ShipVarDefinition),
    MethodDef(ShipMethodDefinition),
    ConstructorDef(ShipConstructorDefinition),
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipMethodDefinition {
    pub method_id: ShipId,
    pub params: Vec<ShipParam>,
    pub return_type: Option<ShipId>,
    pub body: Option<Box<ShipMethodBody>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipParam {
    pub name: ShipId,
    pub var_type: ShipId,
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipMethodBody {
    Body(Box<ShipBody>),
    Expr(Box<ShipExpression>),
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipConstructorDefinition {
    pub params: Vec<ShipParam>,
    pub body: Box<ShipBody>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipVarDefinition {
    pub var_id: ShipId,
    pub expr: Box<ShipExpression>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShipBody {
    pub members: Vec<ShipBodyMember>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipBodyMember {
    VarDef(Box<ShipVarDefinition>),
    Stmt(Box<ShipStatement>),
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipStatement {
    Assign {
        target: Box<ShipExpression>,
        value: Box<ShipExpression>,
    },
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

#[derive(Debug, Clone, Serialize)]
pub enum ShipExpression {
    ConstructorCall { class_id: ShipId, args: Vec<ShipExpression> },
    MemberAccess { expr: Box<ShipExpression>, member_id: ShipId },
    MethodCall { expr: Box<ShipExpression>, args: Vec<ShipExpression> },
    Primary(ShipPrimary),
}

#[derive(Debug, Clone, Serialize)]
pub enum ShipPrimary {
    Int(i32),
    Float(f32),
    This,
    Id(ShipId),
}
