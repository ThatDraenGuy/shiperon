pub mod body;
pub mod expr;
pub mod field;
pub mod registry;
pub mod signature;

use std::rc::Rc;

use crate::{
    analyzer::{
        field::{ClassWithFieldRegistry, FieldError},
        signature::ConsError,
    },
    ast::{ShipId, ShipProgram},
    diagnostics::{Diagnostic, Renderable},
};

use derive_more::{Display, From};
use signature::{ClassDefRegistry, ClassError, ClassSignatureRegistry};

pub struct Analyzer<'src> {
    ast: Rc<ShipProgram<'src>>,
    diagnostics: Vec<Diagnostic<'src>>,
}

impl<'src> Analyzer<'src> {
    pub fn analyze(&mut self) {
        let mut errors = Vec::new();

        let class_defs = ClassDefRegistry::new(&self.ast.classes, &mut errors);
        let class_signatures = ClassSignatureRegistry::new(class_defs, &mut errors);
        let checked_signatures = class_signatures.check_inheritance(&mut errors);
        let with_fields = ClassWithFieldRegistry::new(checked_signatures, &mut errors);
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
}

impl<'src> Renderable<'src> for AnalysisError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        match self {
            AnalysisError::Class(class_error) => class_error.render(src),
            AnalysisError::Field(field_error) => field_error.render(src),
            AnalysisError::General(general_error) => general_error.render(src),
            AnalysisError::Cons(cons_error) => cons_error.render(src),
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
