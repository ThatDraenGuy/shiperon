use std::rc::Rc;

use derive_more::Display;

use crate::{
    analyzer::{
        expr::PrimitiveExpr,
        registry::{ClassId, ConsId, FieldRegistry, FieldRegistryBuilder},
        signature::ClassSignatureRegistry,
    },
    ast::{ShipId, ShipVarDef},
    diagnostics::Renderable,
};

pub enum FieldExpr {
    Primitive(PrimitiveExpr),
    Cons { class: ClassId, cons: ConsId, args: Vec<FieldExpr> },
}

pub struct FieldModel {
    pub field_type: ClassId,
    pub init_expr: FieldExpr,
}

pub type FieldModelRegistry<'src> = FieldRegistry<'src, FieldModel>;
pub type FieldModelRegistryBuilder<'src> = FieldRegistryBuilder<'src, FieldModel>;

#[derive(Debug, Clone, Display)]
pub enum FieldError<'src> {
    #[display("undefined class with name `{cls_name}`")]
    UndefinedClass { cls_name: Rc<ShipId<'src>> },
}

impl<'src> Renderable<'src> for FieldError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
