use crate::analyzer::{
    AnalysisError,
    body::{ConsBodyRegistry, MethodBodyRegistry},
    def::{ClassDefsRegistry, ClassMemberNamesRegistry},
    field::{ClassFields, ClassFieldsRegistry},
    registry::ClassNameRegistry,
    signature::{ClassSignature, ClassSignatureRegistry},
    stdlib::ShipStdLib,
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
    pub fn new<'src>(
        signature: ClassSignature,
        fields: ClassFields,
        method_bodies: MethodBodyRegistry,
        cons_bodies: ConsBodyRegistry,
    ) -> Self {
        let methods = signature.methods.combine(method_bodies, |data, body| {
            data.combine(body, |signature, body| MethodModel::new(signature, body))
        });
        let constructors = signature
            .constructors
            .combine(cons_bodies, |signature, body| ConsModel::new(signature, body));
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

impl ClassModelRegistry {
    pub fn new<'src>(
        stdlib: &ShipStdLib,
        cls_names: &ClassNameRegistry<'src>,
        member_names: &ClassMemberNamesRegistry<'src>,
        signatures: ClassSignatureRegistry,
        fields: ClassFieldsRegistry,
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
                        (
                            cons_id,
                            ConsBody::resolve(
                                stdlib,
                                cls_names,
                                member_names,
                                &signatures,
                                &fields,
                                cls_id,
                                cons_id,
                                cons_def,
                                errors,
                            ),
                        )
                    })
                    .collect();

                let method_bodies = def
                    .methods
                    .iter()
                    .map(|(method_name_id, methods)| {
                        (
                            method_name_id,
                            methods
                                .iter()
                                .map(|(method_overload_id, method_def)| {
                                    (
                                        method_overload_id,
                                        MethodBody::resolve(
                                            stdlib,
                                            cls_names,
                                            member_names,
                                            &signatures,
                                            &fields,
                                            cls_id,
                                            (method_name_id, method_overload_id).into(),
                                            method_def,
                                            errors,
                                        ),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect();
                (cls_id, (method_bodies, cons_bodies))
            })
            .collect();
        signatures.combine(fields, |signature, field| (signature, field)).combine(
            bodies,
            |(signature, fields), (method_bodies, cons_bodies)| {
                ClassModel::new(signature, fields, method_bodies, cons_bodies)
            },
        )
    }
}
