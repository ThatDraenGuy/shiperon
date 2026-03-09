%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%define api.parser.generic {<'src /* 'fix quotes */, S: ByteSource<'src /* 'fix quotes */>>}
%define api.parser.check_debug { self.debug }
%define parse.error custom

%code use {
// dark evil double lexer reacharound combo
use crate::Lexer as AppLexer;
use crate::TokenRegistry as Lexer;
use crate::lexer::Token;
use crate::ByteSource;
use crate::parser::ParserLoc as Loc;
use crate::parser::{value::*, ParserValue as Value};
use crate::ast::*;
}

%code parser_fields {
    lexer: AppLexer<'src /* 'fix quotes */, S>,
    debug: bool,
    pub result: Option<ShipProgram<'src /* 'fix quotes */>>,
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
    class_defs class_def class_id maybe_class_members class_members class_member
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
        class_defs {
            // let program = Value::new_program($<ClassDefs>1);
            // self.result = Some(ShipProgram::from(program));
            // $$ = Value::None;
        }

    class_defs:
        class_def {
            // $$ = Value::new_class_defs(vec![$<ShipClassDefinition>1]);
        } | class_defs class_def {
            // let mut classes = $<ClassDefs>1;
            // classes.push($<ShipClassDefinition>2);
            // $$ = Value::new_class_defs(classes);
        }

    class_def:
        kCLASS class_id kIS maybe_class_members kEND {
            // $$ = Value::new_class_def($<Id>2, Option::None, $<ClassMembers>4);
        } | kCLASS class_id kEXTENDS class_id kIS maybe_class_members kEND {
            // $$ = Value::new_class_def($<Id>2, Option::Some($<Id>4), $<ClassMembers>6);
        }

    class_id:
        general_id {
            // $$ = $1;
        } //TODO generics??

    maybe_class_members:
        %empty {
            // $$ = Value::new_class_members(vec![]);
        } | class_members {
            // $$ = $1;
        }

    class_members:
        class_member {
            // $$ = Value::new_class_members(vec![$<ShipClassMember>1]);
        } | class_members class_member {
            // let mut members = $<ClassMembers>1;
            // members.push($<ShipClassMember>2);
            // $$ = Value::new_class_members(members);
        }

    class_member:
        var_def {
            // $$ = Value::new_class_member_var_def($<ShipVarDefinition>1);
        } | method_def {
            // $$ = Value::new_class_member_method_def($<ShipMethodDefinition>1);
        } | constructor_def {
            // $$ = Value::new_class_member_constructor_def($<ShipConstructorDefinition>1);
        }

    var_def:
        kVAR var_id tCOLON expr {
            // $$ = Value::new_var_def($<Id>2, $<ShipExpression>4);
        }

    var_id:
        general_id {
            // $$ = $1;
        }

    constructor_def:
        kTHIS params kIS body kEND {
            // $$ = Value::new_constructor_def($<Params>2, $<ShipBody>4);
        }

    method_def:
        method_decl {
            // $$ = $1;
        } | method_decl method_body {
            // let method_decl = $<ShipMethodDefinition>1;
            // $$ = Value::MethodDef(ShipMethodDefinition{ body: Option::Some(Box::new($<ShipMethodBody>2)), ..method_decl });
        }

    method_decl:
        kMETHOD method_id params {
            // $$ = Value::new_method_def($<Id>2, $<Params>3, Option::None, Option::None);
        } | kMETHOD method_id params tCOLON type_id {
            // $$ = Value::new_method_def($<Id>2, $<Params>3, Option::Some($<Id>5), Option::None);
        }

    method_id:
        general_id {
            // $$ = $1;
        }

    params:
        tLPAREN maybe_param_array tRPAREN {
            // $$ = $2;
        }

    maybe_param_array:
        %empty {
            // $$ = Value::new_params(vec![]);
        } | param_array {
            // $$ = $1;
        }

    param_array:
        param {
            // $$ = Value::new_params(vec![$<ShipParam>1]);
        } | params tCOMMA param {
            // let mut params = $<Params>1;
            // params.push($<ShipParam>2);
            // $$ = Value::new_params(params);
        }

    param:
        param_id tCOLON type_id {
            // $$ = Value::new_param($<Id>1, $<Id>3);
        }

    param_id:
        general_id {
            // $$ = $1;
        }

    type_id:
        class_id {
            // $$ = $1;
        }

    method_body:
        kIS body kEND {
            // $$ = Value::new_method_body($<ShipBody>2);
        } | tARROW expr {
            // $$ = Value::new_method_body_short($<ShipExpression>2);
        }

    body:
        maybe_body_members {
            // $$ = $1;
        } | maybe_body_members return_stmt {
            // let mut body = $<ShipBody>1;
            // let stmt = $<ShipStatement>2;
            // body.members.push(ShipBodyMember::Stmt(Box::new(stmt))); //TODO
            // $$ = Value::new_body(body);
        }

    maybe_body_members:
        %empty {
            // $$ = Value::new_body(ShipBody{ members: vec![] });
        } | body_members {
            // $$ = $1;
        }
    body_members:
        body_member {
            // $$ = Value::new_body(ShipBody{ members: vec![$<ShipBodyMember>1] });
        } | body_members body_member {
            // let mut body = $<ShipBody>1;
            // body.members.push($<ShipBodyMember>2);
            // $$ = Value::new_body(body);
        }

    body_member:
        var_def {
            // $$ = Value::new_body_member_var_def($<ShipVarDefinition>1);
        } | stmt {
            // $$ = Value::new_body_member_stmt($<ShipStatement>1);
        }

    expr:
        member_access {
            $$ = Value::new_expr(self.src(), ShipExpressionAll::MemberAccess($<ShipMemberAccessExpr>1));
        } | method_call {
            // $$ = $1;
        } | primary {
            // $$ = Value::new_expr(ShipExpressionAll::Primary($<ShipPrimaryAll>1));
        }

    /* constructor_call:
        class_id args {
            $$ = Value::new_expr_constructor_call($<Id>1, $<Args>2);
        } */

    member_access:
        expr tDOT var_id {
            $$ = Value::new_member_access(self.src(), $<ShipExpressionAll>1, $<ShipId>3);
        }

    method_call:
        expr args {
            // $$ = Value::new_expr_method_call($<ShipExpression>1, $<Args>2);
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
            // $$ = $2;
        }

    maybe_args_array:
        %empty {
            // $$ = Value::new_args(vec![]);
        } | args_array {
            // $$ = $1;
        }

    args_array:
        expr {
            // $$ = Value::new_args(vec![$<ShipExpression>1]);
        } | args_array tCOMMA expr {
            // let mut args = $<Args>1;
            // args.push($<ShipExpression>3);
            // $$ = Value::new_args(args);
        }

    stmt:
        assign_stmt {
            // $$ = $1;
        } | while_stmt {
            // $$ = $1;
        } | if_stmt {
            // $$ = $1;
        }

    assign_stmt:
        expr tASSIGN expr {
            // $$ = Value::new_stmt_assign($<ShipExpression>1, $<ShipExpression>3);
        }

    while_stmt:
        kWHILE expr kLOOP body kEND {
            // $$ = Value::new_stmt_while($<ShipExpression>2, $<ShipBody>4);
        }

    if_stmt:
        kIF expr kTHEN body kEND {
            // $$ = Value::new_stmt_if($<ShipExpression>2, $<ShipBody>4, Option::None);
        } | kIF expr kTHEN body kELSE body kEND {
            // $$ = Value::new_stmt_if($<ShipExpression>2, $<ShipBody>4, Option::Some($<ShipBody>6));
        }

    return_stmt:
        kRETURN {
            // $$ = Value::new_stmt_return(Option::None);
        } | kRETURN expr {
            // $$ = Value::new_stmt_return(Option::Some($<ShipExpression>2));
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