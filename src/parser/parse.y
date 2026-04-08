%expect 0

%define api.parser.struct {Parser}
%define api.value.type {Value}

%define api.parser.generic {<'src /* 'fix quotes */, S: ByteSource<'src /* 'fix quotes */>>}
%define api.parser.check_debug { self.config.debug }
%define parse.error custom

%code use {
use std::rc::Rc;
// dark evil double lexer reacharound combo
use crate::Lexer as AppLexer;
use crate::TokenRegistry as Lexer;
use crate::lexer::Token;
use crate::ByteSource;
use crate::parser::{WithParserLoc, value::*, ParseData, ParseError, ParserLoc as Loc, ParserValue as Value};
use crate::ast::*;
use crate::diagnostics::*;
use crate::CompilerConfig;
use crate::ShipFeature;
}

%code parser_fields {
    lexer: AppLexer<'src /* 'fix quotes */, S>,
    config: CompilerConfig,
    result: Option<Rc<ShipProgram<'src /* 'fix quotes */>>>,
    diagnostics: Vec<Diagnostic<'src /* 'fix quotes */>>,
}

%code {
// code
}



/* Bison defarations */
%token
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
    kAS         "as"
    kSUPER      "super"

%token
    tIDENTIFIER "identifier"
    tINTEGER    "integer"
    tFLOAT      "float"
    tSTRING     "string"
    tCHAR       "char"

%token
    tCOLON      ":"
    tARROW      "=>"
    tLPAREN     "("
    tRPAREN     ")"
    tASSIGN     ":="
    tCOMMA      ","
    tDOT        "."

%type <Value> program
    maybe_class_defs class_defs class_def class_id maybe_class_members class_members class_member
    var_def var_id
    constructor_def
    method_def method_decl method_body method_id params maybe_param_array param_array param
    body maybe_body_members body_members body_member
    param_id type_id expr stmt member_access method_call class_cast
    assignable_expr nonassignable_expr callable_expr noncallable_expr non_expr_stmt assign_stmt while_stmt if_stmt if_condition return_stmt
    args maybe_args_array args_array primary primitive
    int float string char super this general_id

%%
    program:
        maybe_class_defs {
            let classes = $<ClassDefsBuilder>1;
            let program = Value::new_program(self.src(), *@1, classes);
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
        } | class_defs error {
            $$ = $1;
        } | error {
            $$ = Value::new_class_defs_builder(vec![]);
        }

    class_def:
        kCLASS class_id kIS maybe_class_members kEND {
            $$ = Value::new_class_def(self.src(), Loc::merge(*@1, *@5), $<ShipId>2, Option::None, $<ClassMembersBuilder>4);
        } | kCLASS class_id kEXTENDS class_id kIS maybe_class_members kEND {
            $$ = Value::new_class_def(self.src(), Loc::merge(*@1, *@7), $<ShipId>2, Option::Some($<ShipId>4), $<ClassMembersBuilder>6);
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
        } | class_members error {
            $$ = $1;
        } | error {
            $$ = Value::new_class_members_builder(vec![]);
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
            $$ = Value::new_var_def(self.src(), Loc::merge(*@1, *@4), $<ShipId>2, $<ShipExprAll>4);
        } | kVAR var_id tASSIGN expr {
            $$ = Value::new_var_def(self.src(), Loc::merge(*@1, *@4), $<ShipId>2, $<ShipExprAll>4);
            self.register_error(*@3, ParseError::AssignOnVarDef);
        }

    var_id:
        general_id {
            $$ = $1;
        }

    constructor_def:
        kTHIS params kIS body kEND {
            let body = ShipBody::new(BodyData{ members: $<BodyBuilder>4 }, Loc::merge(*@3, *@5), self.src());
            $$ = Value::new_cons_def(self.src(), Loc::merge(*@1, *@5), $<ShipParams>2, body);
        } | kTHIS params tCOLON type_id kIS body kEND {
            self.register_error(Loc::merge(*@3, *@4), ParseError::ReturnTypeInCons);
            let body = ShipBody::new(BodyData{ members: $<BodyBuilder>6 }, Loc::merge(*@5, *@7), self.src());
            $$ = Value::new_cons_def(self.src(), Loc::merge(*@1, *@7), $<ShipParams>2, body);
        }

    method_def:
        method_decl {
            $$ = $1;
        } | method_decl method_body {
            $$ = Value::MethodDef(ShipMethodDef::new(MethodDefData{ body: Some($<ShipMethodBodyAll>2), ..$<ShipMethodDef>1.data.clone() }, Loc::merge(*@1, *@2), self.src()));
        }

    method_decl:
        kMETHOD method_id params {
            $$ = Value::new_method_def(self.src(), Loc::merge(*@1, *@3), $<ShipId>2, $<ShipParams>3, Option::None, Option::None);
        } | kMETHOD method_id params tCOLON type_id {
            $$ = Value::new_method_def(self.src(), Loc::merge(*@1, *@5), $<ShipId>2, $<ShipParams>3, Option::Some($<ShipId>5), Option::None);
        }

    method_id:
        general_id {
            $$ = $1;
        }

    params:
        tLPAREN maybe_param_array tRPAREN {
            $$ = Value::new_params(self.src(), Loc::merge(*@1, *@3), $<ParamsBuilder>2);
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
        } | param_array tCOMMA param {
            let mut params = $<ParamsBuilder>1;
            params.push($<ShipParam>3);
            $$ = Value::new_params_builder(params);
        } | param_array error {
            $$ = $1;
        } | error {
            $$ = Value::new_params_builder(vec![]);
        }

    param:
        param_id tCOLON type_id {
            $$ = Value::new_param(self.src(), Loc::merge(*@1, *@3), $<ShipId>1, $<ShipId>3);
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
            let body = ShipBody::new(BodyData{ members: $<BodyBuilder>2 }, Loc::merge(*@1, *@3), self.src());
            $$ = Value::new_method_body(self.src(), ShipMethodBodyAll::Body(body));
        } | tARROW expr {
            $$ = Value::new_method_body(self.src(), ShipMethodBodyAll::Expr($<ShipExprAll>2));
        }

    body:
        maybe_body_members {
            $$ = $1;
        } | maybe_body_members return_stmt {
            let mut body = $<BodyBuilder>1;
            let member = ShipBodyMemberAll::Stmt(ShipStmtAll::Return($<ShipReturnStmt>2));
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
        } | body_members error {
            $$ = $1;
        } | error {
            $$ = Value::new_body_builder(vec![]);
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
            $$ = Value::new_expr(self.src(), ShipExprAll::Call($<ShipCallExpr>1));
        } | primary {
            $$ = Value::new_expr(self.src(), ShipExprAll::Primary($<ShipPrimaryAll>1));
        } | class_cast {
            $$ = Value::new_expr(self.src(), ShipExprAll::ClassCast($<ShipClassCastExpr>1));
        }

    noncallable_expr:
        method_call {
            $$ = Value::None;
        } | primitive {
            $$ = Value::None;
        } | class_cast {
            $$ = Value::None;
        }

    callable_expr:
        member_access {
            $$ = Value::new_callable_expr(self.src(), ShipCallableExprAll::MemberAccess($<ShipMemberAccessExpr>1));
        } | this {
            $$ = Value::new_callable_expr(self.src(), ShipCallableExprAll::This($<ShipThis>1));
        } | super {
            $$ = Value::new_callable_expr(self.src(), ShipCallableExprAll::Super($<ShipSuper>1));
        } | class_id {
            $$ = Value::new_callable_expr(self.src(), ShipCallableExprAll::Cons($<ShipId>1));
        }

    nonassignable_expr:
        method_call {
            $$ = Value::None;
        } | primitive {
            $$ = Value::None;
        } | this {
            $$ = Value::None;
        } | super {
            $$ = Value::None;
        } | class_cast {
            $$ = Value::None;
        }

    assignable_expr:
        member_access {
            $$ = Value::new_assignable_expr(self.src(), ShipAssignableExprAll::MemberAccess($<ShipMemberAccessExpr>1));
        } | var_id {
            $$ = Value::new_assignable_expr(self.src(), ShipAssignableExprAll::Variable($<ShipId>1));
        }

    member_access:
        expr tDOT var_id {
            $$ = Value::new_member_access(self.src(), Loc::merge(*@1, *@3), $<ShipExprAll>1, $<ShipId>3);
        }

    method_call:
        callable_expr args {
            $$ = Value::new_method_call(self.src(), Loc::merge(*@1, *@2), $<ShipCallableExprAll>1, $<ShipArgs>2);
        } | noncallable_expr args {
            self.yyerror(*@1, ParseError::ExprIsNotCallable{ call_args: $<ShipArgs>2 })?;
        }

    class_cast:
        expr kAS class_id {
            self.check_feature(Loc::merge(*@2, *@3), ShipFeature::ClassCasting);
            $$ = Value::new_class_cast(self.src(), Loc::merge(*@1, *@3), $<ShipExprAll>1, $<ShipId>3);
        }

    primitive:
        int {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Int($<ShipInt>1));
        } | float {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Float($<ShipFloat>1));
        } | string {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::String($<ShipString>1));
        } | char {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Char($<ShipChar>1));
        }

    primary:
        primitive {
            $$ = $1;
        } | this {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::This($<ShipThis>1));
        } | var_id {
            $$ = Value::new_primary(self.src(), ShipPrimaryAll::Id($<ShipId>1));
        }

    args:
        tLPAREN maybe_args_array tRPAREN {
            $$ = Value::new_args(self.src(), Loc::merge(*@1, *@3), $<ArgsBuilder>2);
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
        } | args_array error {
            $$ = $1;
        } | error {
            $$ = Value::new_args_builder(vec![]);
        }

    stmt:
        non_expr_stmt {
            $$ = $1;
        }
        | method_call {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::Call($<ShipCallExpr>1));
        }

    non_expr_stmt:
        assign_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::Assign($<ShipAssignStmt>1));
        } | while_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::While($<ShipWhileStmt>1));
        } | if_stmt {
            $$ = Value::new_stmt(self.src(), ShipStmtAll::If($<ShipIfStmt>1));
        }

    assign_stmt:
        assignable_expr tASSIGN expr {
            let left = $<ShipAssignableExprAll>1;
            let right = $<ShipExprAll>3;
            $$ = Value::new_assign_stmt(self.src(), Loc::merge(*@1, *@3), left, right);
        } | nonassignable_expr tASSIGN expr {
            self.yyerror(Loc::merge(*@1, *@3), ParseError::ExprIsNotAssignable{ value: $<ShipExprAll>3 })?;
        }

    while_stmt:
        kWHILE expr kLOOP body kEND {
            let body = ShipBody::new(BodyData{ members: $<BodyBuilder>4 }, Loc::merge(*@3, *@5), self.src());
            $$ = Value::new_while_stmt(self.src(), Loc::merge(*@1, *@5), $<ShipExprAll>2, body);
        }

    if_stmt:
        kIF if_condition kTHEN body kEND {
            let body = ShipBody::new(BodyData{ members: $<BodyBuilder>4 }, Loc::merge(*@3, *@5), self.src());
            $$ = Value::new_if_stmt(self.src(), Loc::merge(*@1, *@5), $<ShipExprAll>2, body, Option::None);
        } | kIF if_condition kTHEN body kELSE body kEND {
            let then_body = ShipBody::new(BodyData{ members: $<BodyBuilder>4 }, Loc::merge(*@3, *@5), self.src());
            let else_body = ShipBody::new(BodyData{ members: $<BodyBuilder>6 }, Loc::merge(*@5, *@7), self.src());
            $$ = Value::new_if_stmt(self.src(), Loc::merge(*@1, *@7), $<ShipExprAll>2, then_body, Option::Some(else_body));
        }

    if_condition:
        expr {
            $$ = $1;
        } | tLPAREN expr tRPAREN {
            self.register_warn(Loc::merge(*@1, *@3), ParseError::UnnecessaryParenthesis);
            $$ = $2;
        }

    return_stmt:
        kRETURN {
            $$ = Value::new_return_stmt(self.src(), *@1, Option::None).0;
        } | kRETURN expr body {
            let (value, return_stmt) = Value::new_return_stmt(self.src(), Loc::merge(*@1, *@2), Option::Some($<ShipExprAll>2));
            $$ = value;

            let afters = $<BodyBuilder>3;
            if !afters.is_empty() {
                self.register_warn(*@3, ParseError::BodyMembersAfterReturn{ return_stmt });
            }
        } | kRETURN non_expr_stmt body {
            let (value, return_stmt) = Value::new_return_stmt(self.src(), *@1, Option::None);
            $$ = value;
            self.register_warn(Loc::merge(*@2, *@3), ParseError::BodyMembersAfterReturn{ return_stmt });
        }

    int:
        tINTEGER {
            $$ = Value::new_int(self.src(), $<Token>1);
        }

    float:
        tFLOAT {
            $$ = Value::new_float(self.src(), $<Token>1);
        }

    string:
        tSTRING {
            self.check_feature(*@1, ShipFeature::String);
            $$ = Value::new_string(self.src(), $<Token>1);
        }

    char:
        tCHAR {
            self.check_feature(*@1, ShipFeature::String);
            $$ = Value::new_char(self.src(), $<Token>1);
        }

    this:
        kTHIS {
            $$ = Value::new_this(self.src(), $<Token>1);
        }

    super:
        kSUPER {
            self.check_feature(*@1, ShipFeature::SuperKeyword);
            $$ = Value::new_super(self.src(), $<Token>1);
        }

    general_id:
        tIDENTIFIER {
            $$ = Value::new_id(self.src(), $<Token>1);
        }
%%

impl<'src /* 'fix quotes */, S: ByteSource<'src /* 'fix quotes */>> Parser<'src /* 'fix quotes */, S> {
    pub fn new(lexer: AppLexer<'src /* 'fix quotes */, S>, config: CompilerConfig) -> Self {
        Self {
            yy_error_verbose: true,
            yynerrs: 0,
            yyerrstatus_: 0,
            yylexer: Lexer{},
            lexer,
            result: None,
            diagnostics: vec![],
            config
        }
    }

    fn src(&self) -> &S {
        &self.lexer.src
    }

    fn next_token(&mut self) -> Token {
        self.lexer.yylex()
    }

    fn check_feature(&mut self, loc: Loc, feature: ShipFeature) {
        if !feature.is_enabled(&self.config.features) {
            self.register_error(loc, ParseError::DisabledFeature(feature));
        }
    }

    fn register_warn(&mut self, loc: Loc, reason: ParseError<'src /* 'fix quotes */>) {
        self.diagnostics.push(Diagnostic{level: ErrorLevel::Warn, loc, reason: reason.into()});
    }

    fn register_error(&mut self, loc: Loc, reason: ParseError<'src /* 'fix quotes */>) {
        self.diagnostics.push(Diagnostic{level: ErrorLevel::Err, loc, reason: reason.into()});
    }

    fn yyerror(&mut self, loc: Loc, reason: ParseError<'src /* 'fix quotes */>) -> Result<i32, ()> {
        self.register_error(loc, reason);
        Err(())
    }

    fn report_syntax_error(&mut self, stack: &YYStack, yytoken: &SymbolKind, loc: YYLoc) {
        let id: usize = yytoken.code().try_into().expect("failed to convert token code into i32, is it too big?");
        self.register_error(loc, ParseError::UnexpectedToken { token_name: Lexer::TOKEN_NAMES[id] });
    }

    pub fn consume_parse(mut self) -> ParseData<'src /* 'fix quotes */, S> {
        self.parse();
        ParseData { program: self.result, diagnostics: self.diagnostics, src: self.lexer.src }
    }
}