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
}

%code parser_fields {
    lexer: AppLexer<S>
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