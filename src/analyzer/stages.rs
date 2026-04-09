use std::rc::Rc;

use crate::{
    analyzer::{
        field::{ClassFields, WithClassFields},
        signature::{ClassSignature, WithClassDef, WithClassSignature},
    },
    ast::ShipClassDef,
};

pub type Stage0<'src> = Rc<ShipClassDef<'src>>;

pub type Stage1<'src> = (Rc<ShipClassDef<'src>>, ClassSignature<'src>);
impl<'src> WithClassDef<'src> for Stage1<'src> {
    #[inline]
    fn class_def(&self) -> &Rc<ShipClassDef<'src>> {
        &self.0
    }
}
impl<'src> WithClassSignature<'src> for Stage1<'src> {
    #[inline]
    fn class_signature(&self) -> &ClassSignature<'src> {
        &self.1
    }
}

pub type Stage2<'src> = (Rc<ShipClassDef<'src>>, ClassSignature<'src>, ClassFields);
impl<'src> WithClassDef<'src> for Stage2<'src> {
    #[inline]
    fn class_def(&self) -> &Rc<ShipClassDef<'src>> {
        &self.0
    }
}
impl<'src> WithClassSignature<'src> for Stage2<'src> {
    #[inline]
    fn class_signature(&self) -> &ClassSignature<'src> {
        &self.1
    }
}
impl<'src> WithClassFields for Stage2<'src> {
    #[inline]
    fn class_fields(&self) -> &ClassFields {
        &self.2
    }
}
