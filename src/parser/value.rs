use std::rc::Rc;

use crate::{
    ByteSource,
    ast::*,
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
    Id(ShipId<'src>),
    Int(ShipInt<'src>),
    Float(ShipFloat<'src>),
    This(ShipThis<'src>),
    Primary(ShipPrimaryAll<'src>),
    ConsCallExpr(ShipConsCallExpr<'src>),
    MethodCallExpr(ShipMethodCallExpr<'src>),
    MemberAccessExpr(ShipMemberAccessExpr<'src>),
    Expr(ShipExpressionAll<'src>),
    // Stmt(ShipStatementAll),
    // Args(Vec<ShipExpressionAll>),
    // Params(Vec<ShipParam>),
    // Param(ShipParam),
    // Body(ShipBody),
    // BodyMember(ShipBodyMemberAll),
    // VarDef(ShipVarDef),
    // MethodBody(ShipMethodBodyAll),
    // MethodDef(ShipMethodDef),
    // ConstructorDef(ShipConsDef),
    // ClassMember(ShipClassMemberAll),
    // ClassMembers(Vec<ShipClassMemberAll>),
    // ClassDef(ShipClassDef),
    // ClassDefs(Vec<ShipClassDef>),
    // Program(ShipProgram),
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
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Id(n) => n,
            other => unreachable!("expected Id, got {:?}", other),
        }
    }
}

impl<'src> ShipInt<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Int(n) => n,
            other => unreachable!("expected Int, got {:?}", other),
        }
    }
}

impl<'src> ShipFloat<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Float(n) => n,
            other => unreachable!("expected Float, got {:?}", other),
        }
    }
}

impl<'src> ShipThis<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::This(n) => n,
            other => unreachable!("expected This, got {:?}", other),
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

impl<'src> ShipMemberAccessExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::MemberAccessExpr(n) => n,
            other => unreachable!("expected MemberAccessExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipMethodCallExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::MethodCallExpr(n) => n,
            other => unreachable!("expected MethodCallExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipConsCallExpr<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::ConsCallExpr(n) => n,
            other => unreachable!("expected ConsCallExpr, got {:?}", other),
        }
    }
}

impl<'src> ShipExpressionAll<'src> {
    pub fn from(value: ParserValue<'src>) -> Self {
        match value {
            ParserValue::Expr(e) => e,
            other => unreachable!("expected Expr, got {:?}", other),
        }
    }
}

// impl ShipStatementAll {
//     pub fn from(value: ParserValue) -> ShipStatementAll {
//         match value {
//             ParserValue::Stmt(s) => s,
//             other => unreachable!("expected Stmt, got {:?}", other),
//         }
//     }
// }

// impl ShipBody {
//     pub fn from(value: ParserValue) -> ShipBody {
//         match value {
//             ParserValue::Body(b) => b,
//             other => unreachable!("expected Body, got {:?}", other),
//         }
//     }
// }

// impl ShipBodyMemberAll {
//     pub fn from(value: ParserValue) -> ShipBodyMemberAll {
//         match value {
//             ParserValue::BodyMember(m) => m,
//             other => unreachable!("expected BodyMember, got {:?}", other),
//         }
//     }
// }

// impl ShipVarDef {
//     pub fn from(value: ParserValue) -> ShipVarDef {
//         match value {
//             ParserValue::VarDef(d) => d,
//             other => unreachable!("expected VarDef, got {:?}", other),
//         }
//     }
// }

// impl ShipMethodBodyAll {
//     pub fn from(value: ParserValue) -> ShipMethodBodyAll {
//         match value {
//             ParserValue::MethodBody(b) => b,
//             other => unreachable!("expected MethodBody, got {:?}", other),
//         }
//     }
// }

// impl ShipParam {
//     pub fn from(value: ParserValue) -> ShipParam {
//         match value {
//             ParserValue::Param(p) => p,
//             other => unreachable!("expected Param, got {:?}", other),
//         }
//     }
// }

// impl ShipMethodDef {
//     pub fn from(value: ParserValue) -> ShipMethodDef {
//         match value {
//             ParserValue::MethodDef(d) => d,
//             other => unreachable!("expected MethodDef, got {:?}", other),
//         }
//     }
// }

// impl ShipConsDef {
//     pub fn from(value: ParserValue) -> ShipConsDef {
//         match value {// impl ShipInt {
//     pub fn from(value: ParserValue) -> ShipInt {
//         match value {
//             ParserValue::Int(n) => n,
//             other => unreachable!("expected Int, got {:?}", other),
//         }
//     }
// }

// impl ShipClassMemberAll {
//     pub fn from(value: ParserValue) -> ShipClassMemberAll {
//         match value {
//             ParserValue::ClassMember(m) => m,
//             other => unreachable!("expected ClassMember, got {:?}", other),
//         }
//     }
// }

// impl ShipClassDef {
//     pub fn from(value: ParserValue) -> ShipClassDef {
//         match value {
//             ParserValue::ClassDef(d) => d,
//             other => unreachable!("expected ClassDef, got {:?}", other),
//         }
//     }
// }

// impl ShipProgram {
//     pub fn from(value: ParserValue) -> ShipProgram {
//         match value {
//             ParserValue::Program(p) => p,
//             other => unreachable!("expected Program, got {:?}", other),
//         }
//     }
// }

// #[allow(non_snake_case)]
// pub mod Args {
//     use super::ParserValue;
//     use crate::ast::ShipExpressionAll;

//     pub fn from(value: ParserValue) -> Vec<ShipExpressionAll> {
//         match value {
//             ParserValue::Args(a) => a,
//             other => unreachable!("expected Args, got {:?}", other),
//         }
//     }
// }

// #[allow(non_snake_case)]
// pub mod Params {
//     use super::ParserValue;
//     use crate::ast::ShipParam;

//     pub fn from(value: ParserValue) -> Vec<ShipParam> {
//         match value {
//             ParserValue::Params(p) => p,
//             other => unreachable!("expected Params, got {:?}", other),
//         }
//     }
// }

// #[allow(non_snake_case)]
// pub mod ClassMembers {
//     use super::ParserValue;
//     use crate::ast::ShipClassMemberAll;

//     pub fn from(value: ParserValue) -> Vec<ShipClassMemberAll> {
//         match value {
//             ParserValue::ClassMembers(m) => m,
//             other => unreachable!("expected ClassMembers, got {:?}", other),
//         }
//     }
// }

// #[allow(non_snake_case)]
// pub mod ClassDefs {
//     use super::ParserValue;
//     use crate::ast::ShipClassDef;

//     pub fn from(value: ParserValue) -> Vec<ShipClassDef> {
//         match value {
//             ParserValue::ClassDefs(d) => d,
//             other => unreachable!("expected ClassDefs, got {:?}", other),
//         }
//     }
// }

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
            TokenValue::String(id) => Self::Id(ShipId::new(IdData { id }, token.loc, src)),
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
            other => unreachable!("expected Integer, got {:?}", other),
        }
    }

    pub fn new_this(src: &impl ByteSource<'src>, token: Token) -> Self {
        Self::This(ShipThis::new(ThisData {}, token.loc, src))
    }

    pub fn new_primary(_src: &impl ByteSource<'src>, primary: ShipPrimaryAll<'src>) -> Self {
        Self::Primary(primary)
    }

    pub fn new_member_access(
        src: &impl ByteSource<'src>,
        expr: ShipExpressionAll<'src>,
        member_id: ShipId<'src>,
    ) -> Self {
        let loc = ParserLoc::merge_from(&expr, &member_id);
        Self::MemberAccessExpr(ShipMemberAccessExpr::new(
            MemberAccessExprData { expr: Rc::new(expr), member_id: Rc::new(member_id) },
            loc,
            src,
        ))
    }

    pub fn new_method_call(
        src: &impl ByteSource<'src>,
        expr: ShipExpressionAll<'src>,
        args: ShipArgs<'src>,
    ) -> Self {
        let loc = ParserLoc::merge_from(&expr, &args);
        Self::MethodCallExpr(ShipMethodCallExpr::new(
            MethodCallExprData { expr: Rc::new(expr), args: Rc::new(args) },
            loc,
            src,
        ))
    }

    pub fn new_expr(_src: &impl ByteSource<'src>, expr: ShipExpressionAll<'src>) -> Self {
        Self::Expr(expr)
    }

    // pub fn new_stmt_assign(target: ShipExpressionAll, value: ShipExpressionAll) -> Self {
    //     Self::Stmt(ShipStatementAll::Assign { target: Box::new(target), value: Box::new(value) })
    // }

    // pub fn new_stmt_while(condition: ShipExpressionAll, body: ShipBody) -> Self {
    //     Self::Stmt(ShipStatementAll::While { condition: Box::new(condition), body: Box::new(body) })
    // }

    // pub fn new_stmt_if(
    //     condition: ShipExpressionAll,
    //     then_body: ShipBody,
    //     else_body: Option<ShipBody>,
    // ) -> Self {
    //     Self::Stmt(ShipStatementAll::If {
    //         condition: Box::new(condition),
    //         then_body: Box::new(then_body),
    //         else_body: else_body.map(Box::new),
    //     })
    // }

    // pub fn new_stmt_return(value: Option<ShipExpressionAll>) -> Self {
    //     Self::Stmt(ShipStatementAll::Return { value: value.map(Box::new) })
    // }

    // pub fn new_args(args: Vec<ShipExpressionAll>) -> Self {
    //     Self::Args(args)
    // }

    // pub fn new_body(members: ShipBody) -> Self {
    //     Self::Body(members)
    // }

    // pub fn new_body_member_stmt(stmt: ShipStatementAll) -> Self {
    //     Self::BodyMember(ShipBodyMemberAll::Stmt(Box::new(stmt)))
    // }

    // pub fn new_body_member_var_def(var_def: ShipVarDef) -> Self {
    //     Self::BodyMember(ShipBodyMemberAll::VarDef(Box::new(var_def)))
    // }

    // pub fn new_var_def(var_id: ShipId, expr: ShipExpressionAll) -> Self {
    //     Self::VarDef(ShipVarDef { var_id, expr: Box::new(expr) })
    // }

    // pub fn new_method_body(body: ShipBody) -> Self {
    //     Self::MethodBody(ShipMethodBodyAll::Body(Box::new(body)))
    // }

    // pub fn new_method_body_short(expr: ShipExpressionAll) -> Self {
    //     Self::MethodBody(ShipMethodBodyAll::Expr(Box::new(expr)))
    // }

    // pub fn new_param(param_id: ShipId, type_id: ShipId) -> Self {
    //     Self::Param(ShipParam { name: param_id, var_type: type_id })
    // }

    // pub fn new_params(params: Vec<ShipParam>) -> Self {
    //     Self::Params(params)
    // }

    // pub fn new_method_def(
    //     method_id: ShipId,
    //     params: Vec<ShipParam>,
    //     return_type: Option<ShipId>,
    //     body: Option<ShipMethodBodyAll>,
    // ) -> Self {
    //     Self::MethodDef(ShipMethodDef { method_id, params, return_type, body: body.map(Box::new) })
    // }

    // pub fn new_constructor_def(params: Vec<ShipParam>, body: ShipBody) -> Self {
    //     Self::ConstructorDef(ShipConsDef { params, body: Box::new(body) })
    // }

    // pub fn new_class_member_var_def(var_def: ShipVarDef) -> Self {
    //     Self::ClassMember(ShipClassMemberAll::VarDef(var_def))
    // }

    // pub fn new_class_member_method_def(method_def: ShipMethodDef) -> Self {
    //     Self::ClassMember(ShipClassMemberAll::MethodDef(method_def))
    // }

    // pub fn new_class_member_constructor_def(constructor_def: ShipConsDef) -> Self {
    //     Self::ClassMember(ShipClassMemberAll::ConsDef(constructor_def))
    // }

    // pub fn new_class_members(members: Vec<ShipClassMemberAll>) -> Self {
    //     Self::ClassMembers(members)
    // }

    // pub fn new_class_def(
    //     class_id: ShipId,
    //     parent_id: Option<ShipId>,
    //     members: Vec<ShipClassMemberAll>,
    // ) -> Self {
    //     Self::ClassDef(ShipClassDef { class_id, parent_id, members })
    // }

    // pub fn new_class_defs(defs: Vec<ShipClassDef>) -> Self {
    //     Self::ClassDefs(defs)
    // }

    // pub fn new_program(classes: Vec<ShipClassDef>) -> Self {
    //     Self::Program(ShipProgram { classes })
    // }
}
