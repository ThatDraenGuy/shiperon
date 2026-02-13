%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%code use {
// all use goes here
}

%code parser_fields {
// all custom parser fields go here
}

%code {
// code
}



/* Bison Declarations */
%token
    tPLUS   "+"
    tMINUS  "-"
    tMUL    "*"
    tDIV    "/"
    tLPAREN "("
    tRPAREN ")"
    tNUM    "`number'"
    tERROR  "controlled YYERROR"
    tABORT  "controlled YYABORT"
    tACCEPT "controlled YYACCEPT"
%type <Number> expr number program

%left "-" "+"
%left "*" "/"

%%
    program: tNUM
%%

impl Parser {
}