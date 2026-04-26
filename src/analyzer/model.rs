use crate::analyzer::{
    AnalysisError,
    body::{ConsBodyRegistry, MethodBodyRegistry},
    def::ClassDefsRegistry,
    field::ClassFields,
    signature::ClassSignature,
};

use super::{
    body::{ConsBody, MethodBody},
    field::FieldModelRegistry,
    registry::{ClassId, ClassRegistry, ConsRegistry, MethodRegistry},
    signature::{MethodSignature, ParamsSignature},
};

pub struct ConsModel {
    pub signature: ParamsSignature,
    pub body: ConsBody,
}
impl ConsModel {
    pub fn new(signature: ParamsSignature, body: ConsBody) -> Self {
        Self { signature, body }
    }
}
pub type ConsModelRegistry = ConsRegistry<ConsModel>;

pub struct MethodModel {
    pub signature: MethodSignature,
    pub body: MethodBody,
}
impl MethodModel {
    pub fn new(signature: MethodSignature, body: MethodBody) -> Self {
        Self { signature, body }
    }
}
pub type MethodModelRegistry = MethodRegistry<MethodModel>;

pub struct ClassModel {
    pub id: ClassId,
    pub parent: ClassId,
    pub constructors: ConsModelRegistry,
    pub methods: MethodModelRegistry,
    pub fields: FieldModelRegistry,
}
impl ClassModel {
    pub fn new(
        signature: ClassSignature,
        fields: ClassFields,
        method_bodies: MethodBodyRegistry,
        cons_bodies: ConsBodyRegistry,
    ) -> Self {
        let methods = signature
            .methods
            .combine(method_bodies, |data, body| data.combine(body, MethodModel::new));
        let constructors = signature.constructors.combine(cons_bodies, ConsModel::new);
        Self {
            id: signature.id,
            parent: signature.parent,
            constructors,
            methods,
            fields: fields.registry,
        }
    }
}
pub type ClassModelRegistry = ClassRegistry<ClassModel>;
pub trait ClassModelCtx {
    fn cls_models(&self) -> &ClassModelRegistry;
}
impl ClassModelCtx for ClassModelRegistry {
    fn cls_models(&self) -> &ClassModelRegistry {
        self
    }
}

impl ClassModelRegistry {
    pub fn new<'a, 'src>(
        ctx: super::BodyResolutionCtx<'a, 'src>,
        defs: &ClassDefsRegistry<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        let bodies = defs
            .iter()
            .map(|(cls_id, def)| {
                let cons_bodies = def
                    .constructors
                    .iter()
                    .map(|(cons_id, cons_def)| {
                        (cons_id, ConsBody::resolve(&ctx, cls_id, cons_id, cons_def, errors))
                    })
                    .collect();

                let method_bodies = def.methods.map_method(|method_id, method_def| {
                    MethodBody::resolve(&ctx, cls_id, method_id, method_def, errors)
                });
                (cls_id, (method_bodies, cons_bodies))
            })
            .collect();
        ctx.cls_signatures.combine(ctx.cls_fields, |signature, field| (signature, field)).combine(
            bodies,
            |(signature, fields), (method_bodies, cons_bodies)| {
                ClassModel::new(signature, fields, method_bodies, cons_bodies)
            },
        )
    }
}
