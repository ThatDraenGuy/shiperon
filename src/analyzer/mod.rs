pub mod body;
pub mod def;
pub mod expr;
pub mod field;
pub mod model;
pub mod registry;
pub mod signature;

use std::rc::Rc;

use crate::{
    ShipStdLib, StdlibCtx,
    analyzer::{
        body::BodyError,
        def::{ClassMemberNamesCtx, ClassMemberNamesRegistry, ClassNamesCtx, init_cls_registry},
        field::{ClassFieldsCtx, ClassFieldsRegistry, FieldError, init_class_fields_registry},
        model::{ClassModelCtx, ClassModelRegistry},
        registry::ClassNameRegistry,
        signature::{
            ClassSignatureCtx, ClassSignatureRegistry, ConsError, MethodError,
            init_cls_signature_registry,
        },
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
    pub fn analyze(
        &mut self,
        stdlib: &ShipStdLib,
    ) -> (
        ClassNameRegistry<'src>,
        ClassMemberNamesRegistry<'src>,
        ClassModelRegistry,
        Vec<Diagnostic<'src>>,
    ) {
        let mut errors = Vec::new();

        let (cls_names, member_names, defs) = init_cls_registry(&self.ast.classes, &mut errors);
        let cls_signatures = init_cls_signature_registry(
            &SignatureResolutionCtx { stdlib, cls_names: &cls_names, member_names: &member_names },
            &defs,
            &mut errors,
        );
        let cls_fields = init_class_fields_registry(
            &FieldResolutionCtx { stdlib, cls_signatures: &cls_signatures, cls_names: &cls_names },
            &defs,
            &mut errors,
        );

        let result = ClassModelRegistry::new(
            BodyResolutionCtx {
                stdlib,
                cls_names: &cls_names,
                cls_member_names: &member_names,
                cls_signatures,
                cls_fields,
            },
            &defs,
            &mut errors,
        );
        (cls_names, member_names, result, errors.into_iter().map(|e| e.into()).collect())
    }
}

pub struct SignatureResolutionCtx<'a, 'src> {
    stdlib: &'a ShipStdLib,
    cls_names: &'a ClassNameRegistry<'src>,
    member_names: &'a ClassMemberNamesRegistry<'src>,
}
impl<'a, 'src> StdlibCtx for SignatureResolutionCtx<'a, 'src> {
    fn stdlib(&self) -> &ShipStdLib {
        self.stdlib
    }
}
impl<'a, 'src> ClassNamesCtx<'src> for SignatureResolutionCtx<'a, 'src> {
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        self.cls_names
    }
}
impl<'a, 'src> ClassMemberNamesCtx<'src> for SignatureResolutionCtx<'a, 'src> {
    fn member_names(&self) -> &ClassMemberNamesRegistry<'src> {
        self.member_names
    }
}

struct FieldResolutionCtx<'a, 'src> {
    stdlib: &'a ShipStdLib,
    cls_signatures: &'a ClassSignatureRegistry,
    cls_names: &'a ClassNameRegistry<'src>,
}
impl<'a, 'src> StdlibCtx for FieldResolutionCtx<'a, 'src> {
    fn stdlib(&self) -> &ShipStdLib {
        self.stdlib
    }
}
impl<'a, 'src> ClassSignatureCtx for FieldResolutionCtx<'a, 'src> {
    fn signatures(&self) -> &ClassSignatureRegistry {
        self.cls_signatures
    }
}
impl<'a, 'src> ClassNamesCtx<'src> for FieldResolutionCtx<'a, 'src> {
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        self.cls_names
    }
}

pub struct BodyResolutionCtx<'a, 'src> {
    stdlib: &'a ShipStdLib,
    cls_names: &'a ClassNameRegistry<'src>,
    cls_member_names: &'a ClassMemberNamesRegistry<'src>,
    cls_signatures: ClassSignatureRegistry,
    cls_fields: ClassFieldsRegistry,
}
impl<'a, 'src> StdlibCtx for BodyResolutionCtx<'a, 'src> {
    fn stdlib(&self) -> &ShipStdLib {
        self.stdlib
    }
}
impl<'a, 'src> ClassSignatureCtx for BodyResolutionCtx<'a, 'src> {
    fn signatures(&self) -> &ClassSignatureRegistry {
        &self.cls_signatures
    }
}
impl<'a, 'src> ClassNamesCtx<'src> for BodyResolutionCtx<'a, 'src> {
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        self.cls_names
    }
}
impl<'a, 'src> ClassMemberNamesCtx<'src> for BodyResolutionCtx<'a, 'src> {
    fn member_names(&self) -> &ClassMemberNamesRegistry<'src> {
        self.cls_member_names
    }
}
impl<'a, 'src> ClassFieldsCtx for BodyResolutionCtx<'a, 'src> {
    fn cls_fields(&self) -> &ClassFieldsRegistry {
        &self.cls_fields
    }
}

pub struct ShipContext<'src> {
    stdlib: ShipStdLib,
    cls_names: ClassNameRegistry<'src>,
    cls_member_names: ClassMemberNamesRegistry<'src>,
    cls_models: ClassModelRegistry,
}
impl<'src> StdlibCtx for ShipContext<'src> {
    fn stdlib(&self) -> &ShipStdLib {
        &self.stdlib
    }
}
impl<'src> ClassNamesCtx<'src> for ShipContext<'src> {
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        &self.cls_names
    }
}
impl<'src> ClassMemberNamesCtx<'src> for ShipContext<'src> {
    fn member_names(&self) -> &ClassMemberNamesRegistry<'src> {
        &self.cls_member_names
    }
}
impl<'src> ClassModelCtx for ShipContext<'src> {
    fn cls_models(&self) -> &ClassModelRegistry {
        &self.cls_models
    }
}

pub trait ShipCtx<'src>:
    StdlibCtx + ClassNamesCtx<'src> + ClassMemberNamesCtx<'src> + ClassModelCtx
{
}
impl<'src, Ctx: StdlibCtx + ClassNamesCtx<'src> + ClassMemberNamesCtx<'src> + ClassModelCtx>
    ShipCtx<'src> for Ctx
{
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
