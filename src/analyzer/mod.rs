pub mod body;
pub mod expr;
pub mod field;
pub mod model;
pub mod registry;
pub mod signature;
pub mod stages;

use std::rc::Rc;

use crate::{
    analyzer::{
        body::BodyError,
        field::{ClassWithFieldRegistry, FieldError},
        model::ClassModelRegistry,
        signature::{ConsError, MethodError},
    },
    ast::{ShipId, ShipProgram},
    diagnostics::{Diagnostic, Renderable},
};

use derive_more::{Display, From};
use signature::{ClassDefRegistry, ClassError, ClassSignatureRegistry};

pub struct Analyzer<'src> {
    ast: Rc<ShipProgram<'src>>,
}

impl<'src> Analyzer<'src> {
    pub fn new(ast: Rc<ShipProgram<'src>>) -> Self {
        Self { ast }
    }
    pub fn analyze(&mut self) -> (ClassModelRegistry, Vec<Diagnostic<'src>>) {
        let mut errors = Vec::new();

        let class_defs = ClassDefRegistry::new(&self.ast.classes, &mut errors);
        let class_signatures = ClassSignatureRegistry::new(class_defs, &mut errors);
        let checked_signatures = class_signatures.check_inheritance(&mut errors);
        let with_fields = ClassWithFieldRegistry::new(checked_signatures, &mut errors);
        let result = ClassModelRegistry::new(with_fields, &mut errors);
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
        todo!()
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
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}

impl<'src> From<GeneralError<'src>> for Diagnostic<'src> {
    fn from(value: GeneralError<'src>) -> Self {
        todo!()
    }
}
