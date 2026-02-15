%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%define parse.error custom

%code use {
// all use goes here
use crate::Lexer;
use crate::lexer::Token;
use crate::parser::ParserLoc as Loc;
use crate::parser::ParserValue as Value;
}

%code parser_fields {
// all custom parser fields go here
}

%code {
// code
}



/* Bison Declarations */
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

%type <Number> expr number program


%%
    program: tINTEGER
%%

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            yy_error_verbose: true,
            yynerrs: 0,
            yyerrstatus_: 0,
            yylexer: lexer,
        }
    }

    fn next_token(&mut self) -> Token {
        self.yylexer.yylex()
    }

    fn report_syntax_error(&self, stack: &YYStack, yytoken: &SymbolKind, loc: YYLoc) {
        eprintln!("report_syntax_error: {:#?} {:?} {:?}", stack, yytoken, loc)
    }
}