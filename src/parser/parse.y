%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%define api.parser.generic {<'src /* 'fix quotes */, S: ByteSource<'src /* 'fix quotes */>>}
%define api.parser.check_debug { self.debug }
%define parse.error custom

%code use {
use std::rc::Rc;
// dark evil double lexer reacharound combo
use crate::Lexer as AppLexer;
use crate::TokenRegistry as Lexer;
use crate::lexer::Token;
use crate::ByteSource;
use crate::parser::{WithParserLoc, value::*, ParserLoc as Loc, ParserValue as Value};
use crate::ast::*;
}

%code parser_fields {
    lexer: AppLexer<'src /* 'fix quotes */, S>,
    debug: bool,
    pub result: Option<Rc<ShipProgram<'src /* 'fix quotes */>>>,
}

%code {
// code
}



/* Bison defarations */
%token <token>
    kCLASS      "class"
    kEXTENDS    "extends"
    kIS         "is"
    kEND        "end"
    kVAR        "var"
    kMETHOD     "method"
    kTHIS       "this"
    kWHILE      "while"
    kLOOP       "loop"
    kIF         "if"
    kTHEN       "then"
    kELSE       "else"
    kRETURN     "return"

%token
    tIDENTIFIER "identifier"
    tINTEGER    "integer"
    tFLOAT      "float"

%token <token> 
    tCOLON      ":"
    tARROW      "=>"
    tLPAREN     "("
    tRPAREN     ")"
    tASSIGN     ":="
    tCOMMA      ","
    tDOT        "."

%type <node> program
    maybe_class_defs class_defs class_def class_id maybe_class_members class_members class_member
    var_def var_id
    constructor_def
    method_def method_decl method_body method_id params maybe_param_array param_array param
    body maybe_body_members body_members body_member
    param_id type_id expr stmt constructor_call member_access method_call
    assign_stmt while_stmt if_stmt return_stmt
    args maybe_args_array args_array primary
    int float this general_id

%%
    program:
        maybe_class_defs {
            let classes = $<ClassDefsBuilder>1;
            let loc = match (classes.first(), classes.last()) {
                (None, None) => Loc{ begin: 0, end: 0 },
                (Some(first), _) => first.loc(),
                (Some(first), Some(last)) => Loc::merge_from(first.as_ref(), last.as_ref()),
                (_, _) => unreachable!("literally how"),
            };
            let program = Value::new_program(self.src(), loc, classes);
            self.result = Some(ShipProgram::from(program));
            $$ = Value::None;
        }

    maybe_class_defs:
        %empty {
            $$ = Value::new_class_defs_builder(vec![]);
        } | class_defs {
            $$ = $1;
        }

    class_defs:
        class_def {
            $$ = Value::new_class_defs_builder(vec![$<ShipClassDef>1]);
        } | class_defs class_def {
            let mut classes = $<ClassDefsBuilder>1;
            classes.push($<ShipClassDef>2);
            $$ = Value::new_class_defs_builder(classes);
        }

    class_def:
        kCLASS class_id kIS maybe_class_members kEND {
            let kclass = $<Token>1;
            let kend = $<Token>5;

            let loc = Loc::merge_from(&kclass, &kend);
            $$ = Value::new_class_def(self.src(), loc, $<ShipId>2, Option::None, $<ClassMembersBuilder>4);
        } | kCLASS class_id kEXTENDS class_id kIS maybe_class_members kEND {
            let kclass = $<Token>1;
            let kend = $<Token>7;

            let loc = Loc::merge_from(&kclass, &kend);
            $$ = Value::new_class_def(self.src(), loc, $<ShipId>2, Option::Some($<ShipId>4), $<ClassMembersBuilder>4);
        }

    class_id:
        general_id {
            $$ = $1;
        } //TODO generics??

    maybe_class_members:
        %empty {
            $$ = Value::new_class_members_builder(vec![]);
        } | class_members {
            $$ = $1;
        }

    class_members:
        class_member {
            $$ = Value::new_class_members_builder(vec![$<ShipClassMemberAll>1]);
        } | class_members class_member {
            let mut members = $<ClassMembersBuilder>1;
            members.push($<ShipClassMemberAll>2);
            $$ = Value::new_class_members_builder(members);
        }

    class_member:
        var_def {
            $$ = Value::new_class_member(self.src(), ShipClassMemberAll::VarDef($<ShipVarDef>1));
        } | method_def {
            $$ = Value::new_class_member(self.src(), ShipClassMemberAll::MethodDef($<ShipMethodDef>1));
        } | constructor_def {
            $$ = Value::new_class_member(self.src(), ShipClassMemberAll::ConsDef($<ShipConsDef>1));
        }

    var_def:
        kVAR var_id tCOLON expr {
            let kvar = $<Token>1;
            let expr = $<ShipExprAll>4;
            let loc = Loc::merge_from(&kvar, &expr);
            $$ = Value::new_var_def(self.src(), loc, $<ShipId>2, expr);
        }

    var_id:
        general_id {
            $$ = $1;
        }

    constructor_def:
        kTHIS params kIS body kEND {
            let kthis = $<Token>1;
            let params = $<ShipParams>2;
            let kis = $<Token>3;
            let members = $<BodyBuilder>4;
            let kend = $<Token>5;

            let body_loc = Loc::merge_from(&kis, &kend);
            let body = ShipBody::new(BodyData{ members }, body_loc, self.src());

            let loc = Loc::merge_from(&kthis, &kend);
            $$ = Value::new_cons_def(self.src(), loc, params, body);
        }

    method_def:
        method_decl {
            $$ = $1;
        } | method_decl method_body {
            let method_decl = $<ShipMethodDef>1;
            let method_body = $<ShipMethodBodyAll>2;
            let loc = Loc::merge_from(method_decl.as_ref(), &method_body);
            $$ = Value::MethodDef(ShipMethodDef::new(MethodDefData{ body: Some(method_body), ..method_decl.data.clone() }, loc, self.src()));
        }

    method_decl:
        kMETHOD method_id params {
            let kmethod = $<Token>1;
            let method_id = $<ShipId>2;
            let params = $<ShipParams>3;
            let loc = Loc::merge_from(&kmethod, params.as_ref());
            $$ = Value::new_method_def(self.src(), loc, method_id, params, Option::None, Option::None);
        } | kMETHOD method_id params tCOLON type_id {
            let kmethod = $<Token>1;
            let method_id = $<ShipId>2;
            let params = $<ShipParams>3;
            let type_id = $<ShipId>5;
            let loc = Loc::merge_from(&kmethod, type_id.as_ref());
            $$ = Value::new_method_def(self.src(), loc, method_id, params, Option::Some(type_id), Option::None);
        }

    method_id:
        general_id {
            $$ = $1;
        }

    params:
        tLPAREN maybe_param_array tRPAREN {
            let loc = Loc::merge_from(&$<Token>1, &$<Token>3);
            $$ = Value::new_params(self.src(), loc, $<ParamsBuilder>2);
        }

    maybe_param_array:
        %empty {
            $$ = Value::new_params_builder(vec![]);
        } | param_array {
            $$ = $1;
        }

    param_array:
        param {
            $$ = Value::new_params_builder(vec![$<ShipParam>1]);
        } | params tCOMMA param {
            let mut params = $<ParamsBuilder>1;
            params.push($<ShipParam>3);
            $$ = Value::new_params_builder(params);
        }

    param:
        param_id tCOLON type_id {
            let param_id = $<ShipId>1;
            let type_id = $<ShipId>3;
            let loc = Loc::merge_from(param_id.as_ref(), type_id.as_ref());
            $$ = Value::new_param(self.src(), loc, param_id, type_id);
        }

    param_id:
        general_id {
            $$ = $1;
        }

    type_id:
        class_id {
            $$ = $1;
        }

    method_body:
        kIS body kEND {
            let kis = $<Token>1;
            let kend = $<Token>3;

            let members = $<BodyBuilder>2;
            let body_loc = Loc::merge_from(&kis, &kend);
            let body = ShipBody::new(BodyData{ members }, body_loc, self.src());

            $$ = Value::new_method_body(self.src(), ShipMethodBodyAll::Body(body));
        } | tARROW expr {
            $$ = Value::new_method_body(self.src(), ShipMethodBodyAll::Expr($<ShipExprAll>2));
        }

    body:
        maybe_body_members {
            $$ = $1;
        } | maybe_body_members return_stmt {
            let mut body = $<BodyBuilder>1;
            let return_stmt = $<ShipReturnStmt>2;
            let stmt = ShipStmtAll::Return(return_stmt);
            let member = ShipBodyMemberAll::Stmt(stmt);
            body.push(member);
            $$ = Value::new_body_builder(body);
        }

    maybe_body_members:
        %empty {
            $$ = Value::new_body_builder(vec![]);
        } | body_members {
            $$ = $1;
        }
    body_members:
        body_member {
            $$ = Value::new_body_builder(vec![$<ShipBodyMemberAll>1]);
        } | body_members body_member {
            let mut body = $<BodyBuilder>1;
            body.push($<ShipBodyMemberAll>2);
            $$ = Value::new_body_builder(body);
        }

    body_member:
        var_def {
            $$ = Value::new_body_member(self.src(), ShipBodyMemberAll::VarDef($<ShipVarDef>1));
        } | stmt {
            $$ = Value::new_body_member(self.src(), ShipBodyMemberAll::Stmt($<ShipStmtAll>1));
        }

    expr:
        member_access {
            $$ = Value::new_expr(self.src(), ShipExprAll::MemberAccess($<ShipMemberAccessExpr>1));
        } | method_call {
            $$ = Value::new_expr(self.src(), ShipExprAll::MethodCall($<ShipMethodCallExpr>1));
        } | primary {
            $$ = Value::new_expr(self.src(), ShipExprAll::Primary($<ShipPrimaryAll>1));
        }

    /* constructor_call:
        class_id args {
            $$ = Value::new_expr_constructor_call($<Id>1, $<Args>2);
        } */

    member_access:
        expr tDOT var_id {
            $$ = Value::new_member_access(self.src(), $<ShipExprAll>1, $<ShipId>3);
        }

    method_call:
        expr args {
            $$ = Value::new_method_call(self.src(), $<ShipExprAll>1, $<ShipArgs>2);
        }

    primary:
        int {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Int($<ShipInt>1));
        } | float {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Float($<ShipFloat>1));
        } | this {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::This($<ShipThis>1));
        } | general_id {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Id($<ShipId>1));
        }

    args:
        tLPAREN maybe_args_array tRPAREN {
            let loc = Loc::merge_from(&$<Token>1, &$<Token>3);
            $$ = Value::new_args(self.src(), loc, $<ArgsBuilder>2);
        }

    maybe_args_array:
        %empty {
            $$ = Value::new_args_builder(vec![]);
        } | args_array {
            $$ = $1;
        }

    args_array:
        expr {
            $$ = Value::new_args_builder(vec![$<ShipExprAll>1]);
        } | args_array tCOMMA expr {
            let mut args = $<ArgsBuilder>1;
            args.push($<ShipExprAll>3);
            $$ = Value::new_args_builder(args);
        }

    stmt:
        assign_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::Assign($<ShipAssignStmt>1));
        } | while_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::While($<ShipWhileStmt>1));
        } | if_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::If($<ShipIfStmt>1));
        }

    assign_stmt:
        expr tASSIGN expr {
            let left = $<ShipExprAll>1;
            let right = $<ShipExprAll>3;
            let loc = Loc::merge_from(&left, &right);
            $$ = Value::new_assign_stmt(self.src(), loc, left, right);
        }

    while_stmt:
        kWHILE expr kLOOP body kEND {
            let kwhile = $<Token>1;
            let kloop = $<Token>3;
            let kend = $<Token>5;

            let members = $<BodyBuilder>4;
            let body_loc = Loc::merge_from(&kloop, &kend);
            let body = ShipBody::new(BodyData{ members }, body_loc, self.src());

            let loc = Loc::merge_from(&kwhile, &kend);
            $$ = Value::new_while_stmt(self.src(), loc, $<ShipExprAll>2, body);
        }

    if_stmt:
        kIF expr kTHEN body kEND {
            let kif = $<Token>1;
            let kthen = $<Token>3;
            let kend = $<Token>5;

            let members = $<BodyBuilder>4;
            let body_loc = Loc::merge_from(&kthen, &kend);
            let body = ShipBody::new(BodyData{ members }, body_loc, self.src());

            let loc = Loc::merge_from(&kif, &kend);
            $$ = Value::new_if_stmt(self.src(), loc, $<ShipExprAll>2, body, Option::None);
        } | kIF expr kTHEN body kELSE body kEND {
            let kif = $<Token>1;
            let kthen = $<Token>3;
            let kelse = $<Token>5;
            let kend = $<Token>7;

            let then_members = $<BodyBuilder>4;
            let then_body_loc = Loc::merge_from(&kthen, &kelse);
            let then_body = ShipBody::new(BodyData{ members: then_members }, then_body_loc, self.src());

            let else_members = $<BodyBuilder>6;
            let else_body_loc = Loc::merge_from(&kelse, &kend);
            let else_body = ShipBody::new(BodyData{ members: else_members }, else_body_loc, self.src());

            let loc = Loc::merge_from(&kif, &kend);
            $$ = Value::new_if_stmt(self.src(), loc, $<ShipExprAll>2, then_body, Option::Some(else_body));
        }

    return_stmt:
        kRETURN {
            let loc = $<Token>1.loc();
            $$ = Value::new_return_stmt(self.src(), loc, Option::None);
        } | kRETURN expr {
            let kreturn = $<Token>1;
            let expr = $<ShipExprAll>2;
            let loc = Loc::merge_from(&kreturn, &expr);
            $$ = Value::new_return_stmt(self.src(), loc, Option::Some(expr));
        }

    int:
        tINTEGER {
            $$ = Value::new_int(self.src(), $<Token>1);
        }

    float:
        tFLOAT {
            $$ = Value::new_float(self.src(), $<Token>1);
        }

    this:
        kTHIS {
            $$ = Value::new_this(self.src(), $<Token>1);
        }

    general_id:
        tIDENTIFIER {
            $$ = Value::new_id(self.src(), $<Token>1);
        }
%%

impl<'src /* 'fix quotes */, S: ByteSource<'src /* 'fix quotes */>> Parser<'src /* 'fix quotes */, S> {
    pub fn new(lexer: AppLexer<'src /* 'fix quotes */, S>, debug: bool) -> Self {
        Self {
            yy_error_verbose: true,
            yynerrs: 0,
            yyerrstatus_: 0,
            yylexer: Lexer{},
            lexer,
            result: None,
            debug,
        }
    }

    fn src(&self) -> &S {
        &self.lexer.src
    }

    fn next_token(&mut self) -> Token {
        self.lexer.yylex()
    }

    fn report_syntax_error(&self, stack: &YYStack, yytoken: &SymbolKind, loc: YYLoc) {
        eprintln!("report_syntax_error: {:#?} {:?} {:?}", stack, yytoken, loc)
    }
}