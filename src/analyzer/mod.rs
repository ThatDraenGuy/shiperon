pub mod body;
pub mod def;
pub mod expr;
pub mod field;
pub mod model;
pub mod registry;
pub mod signature;
pub mod stdlib;

use std::rc::Rc;

use crate::{
    analyzer::{
        body::BodyError,
        def::init_cls_registry,
        field::{FieldError, init_class_fields_registry},
        model::ClassModelRegistry,
        signature::{ConsError, MethodError, init_cls_signature_registry},
        stdlib::ShipStdLib,
    },
    ast::{ShipId, ShipProgram},
    diagnostics::{Diagnostic, ErrorLevel, Renderable},
    parser::WithParserLoc,
};

use derive_more::{Display, From};
use signature::ClassError;

pub struct Analyzer<'src> {
    ast: Rc<ShipProgram<'src>>,
}

impl<'src> Analyzer<'src> {
    pub fn new(ast: Rc<ShipProgram<'src>>) -> Self {
        Self { ast }
    }
    pub fn analyze(&mut self, stdlib: &ShipStdLib) -> (ClassModelRegistry, Vec<Diagnostic<'src>>) {
        let mut errors = Vec::new();

        let (cls_names, member_names, defs) = init_cls_registry(&self.ast.classes, &mut errors);
        let cls_signatures = init_cls_signature_registry(&cls_names, &defs, &mut errors);
        let cls_fields =
            init_class_fields_registry(stdlib, &cls_signatures, &cls_names, &defs, &mut errors);

        let result = ClassModelRegistry::new(
            stdlib,
            &cls_names,
            &member_names,
            cls_signatures,
            cls_fields,
            &defs,
            &mut errors,
        );
        (result, errors.into_iter().map(|e| e.into()).collect())
    }
}

#[derive(Debug, Clone, Display, From)]
pub enum AnalysisError<'src> {
    #[display("{_0}")]
    General(GeneralError<'src>),
    #[display("{_0}")]
    Class(ClassError<'src>),
    #[display("{_0}")]
    Field(FieldError<'src>),
    #[display("{_0}")]
    Cons(ConsError<'src>),
    #[display("{_0}")]
    Body(BodyError<'src>),
    #[display("{_0}")]
    Method(MethodError<'src>),
}

impl<'src> From<AnalysisError<'src>> for Diagnostic<'src> {
    fn from(value: AnalysisError<'src>) -> Self {
        Diagnostic {
            level: ErrorLevel::Err,
            loc: match &value {
                AnalysisError::General(e) => e.loc(),
                AnalysisError::Class(e) => e.loc(),
                AnalysisError::Field(e) => e.loc(),
                AnalysisError::Cons(e) => e.loc(),
                AnalysisError::Body(e) => e.loc(),
                AnalysisError::Method(e) => e.loc(),
            },
            reason: value.into(),
        }
    }
}

impl<'src> Renderable<'src> for AnalysisError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        match self {
            AnalysisError::Class(class_error) => class_error.render(src),
            AnalysisError::Field(field_error) => field_error.render(src),
            AnalysisError::General(general_error) => general_error.render(src),
            AnalysisError::Cons(cons_error) => cons_error.render(src),
            AnalysisError::Body(body_error) => body_error.render(src),
            AnalysisError::Method(method_error) => method_error.render(src),
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum GeneralError<'src> {
    #[display("undefined class with name `{cls_name}`")]
    UndefinedClass { cls_name: Rc<ShipId<'src>> },
}

impl<'src> Renderable<'src> for GeneralError<'src> {
    fn render(&self, _src: &impl crate::ByteSource<'src>) -> String {
        match self {
            GeneralError::UndefinedClass { cls_name } => {
                format!("Class with name `{}` was not found", cls_name.id)
            },
        }
    }
}

impl<'src> WithParserLoc for GeneralError<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            GeneralError::UndefinedClass { cls_name } => cls_name.loc(),
        }
    }
}
