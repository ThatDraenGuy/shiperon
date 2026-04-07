pub mod registry;
pub mod signature;

use std::rc::Rc;

use crate::{
    ast::ShipProgram,
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

        //1. class registry + inheritance. Errors: circular dependency, undefiend parent
        let class_defs = ClassDefRegistry::new(&self.ast.classes, &mut errors);
        let class_signatures = ClassSignatureRegistry::new(class_defs, &mut errors);

        //2. class view (field names & method+cons types). Errors: circular field dependency, duplicate fields/methods/cons
        //3. expressions (fields)???
    }
}

#[derive(Debug, Clone, Display, From)]
pub enum AnalysisError<'src> {
    Class(ClassError<'src>),
}

impl<'src> Renderable<'src> for AnalysisError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        match self {
            AnalysisError::Class(class_error) => class_error.render(src),
        }
    }
}
