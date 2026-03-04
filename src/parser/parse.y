%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%define api.parser.generic {<S: ByteSource>}
%define parse.error custom

%code use {
// dark evil double lexer reacharound combo
use crate::Lexer as AppLexer;
use crate::TokenRegistry as Lexer;
use crate::lexer::Token;
use crate::lexer::ByteSource;
use crate::parser::ParserLoc as Loc;
use crate::parser::ParserValue as Value;
use crate::ast::*;
}

%code parser_fields {
    lexer: AppLexer<S>
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
    class_defs class_def class_id class_members class_member
    var_def var_id
    constructor_def
    method_def method_decl method_body method_id params param_array param
    body body_members body_member
    param_id type_id expr stmt constructor_call member_access
    assign_stmt while_stmt if_stmt return_stmt
    args args_array primary

%%
    program:
        class_defs {

        }

    class_defs:
        class_def {

        } | class_defs class_def {

        }

    class_def:
        kCLASS class_id kIS class_members kEND {

        }

    class_id:
        tIDENTIFIER {

        }

    class_members:
        class_member {

        } | class_members class_member {

        }

    class_member:
        var_def {

        } | method_def {

        } | constructor_def {

        }

    var_def:
        kVAR var_id tCOLON expr {

        }

    var_id:
        tIDENTIFIER {

        }

    constructor_def:
        kTHIS params kIS body kEND {

        }

    method_def:
        method_decl {

        } | method_decl method_body {

        }

    method_decl:
        kMETHOD method_id params {

        } | kMETHOD method_id params tCOLON type_id {
            
        }

    method_id:
        tIDENTIFIER {

        }

    params:
        tLPAREN tRPAREN {

        } | tLPAREN param_array tRPAREN {

        }

    param_array:
        param {

        } | params tCOMMA param {

        }

    param:
        param_id tCOLON type_id {

        }

    param_id:
        tIDENTIFIER {

        }

    type_id:
        tIDENTIFIER {

        }

    method_body:
        kIS body kEND {

        } | tARROW expr {

        }

    body:
        body_members {

        } | body_members return_stmt {
            
        }

    body_members:
        body_member {

        } | body_members body_member {

        }

    body_member:
        var_def {

        } | stmt {

        }

    expr:
        constructor_call {

        } | member_access {
            $$ = $1;
        } | method_call {

        } | primary {
            $$ = Value::new_expr_primary($<ShipPrimary>1);
        }

    constructor_call:
        class_id args {
            $$ = Value::new_expr_constructor_call($<Token>1, $<Args>2);
        }

    member_access:
        expr tDOT var_id {
            $$ = Value::new_expr_member_access($<ShipExpression>1, $<Token>2);
        }

    method_call:
        expr args {

        }

    primary:
        tINTEGER {
            $$ = Value::new_primary_int($<Token>1);
        } | tFLOAT {
            $$ = Value::new_primary_float($<Token>1);
        } | kTHIS {
            $$ = Value::new_primary_this();
        }

    args:
        tLPAREN tRPAREN {

        } | tLPAREN args_array tRPAREN {

        }

    args_array:
        expr {
            $$ = Value::new_args();
            
        } | args_array tCOMMA expr {

        }

    stmt:
        assign_stmt {
        
        } | while_stmt {

        } | if_stmt {

        }

    assign_stmt:
        var_id tASSIGN expr {

        }

    while_stmt:
        kWHILE expr kLOOP body kEND {

        }

    if_stmt:
        kIF expr kTHEN body kEND {

        } | kIF expr kTHEN body kELSE body kEND {

        }

    return_stmt:
        kRETURN {

        } | kRETURN expr {

        }
%%

impl<S: ByteSource> Parser<S> {
    pub fn new(lexer: AppLexer<S>) -> Self {
        Self {
            yy_error_verbose: true,
            yynerrs: 0,
            yyerrstatus_: 0,
            yylexer: Lexer{},
            lexer
        }
    }

    fn next_token(&mut self) -> Token {
        self.lexer.yylex()
    }

    fn report_syntax_error(&self, stack: &YYStack, yytoken: &SymbolKind, loc: YYLoc) {
        eprintln!("report_syntax_error: {:#?} {:?} {:?}", stack, yytoken, loc)
    }
}