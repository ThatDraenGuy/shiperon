use crate::analyzer::{
    AnalysisError,
    body::{ConsBodyRegistry, MethodBodyRegistry},
    field::{ClassFields, ClassWithFieldRegistry},
    signature::{ClassSignature, WithClassSignature, WithParamsSignature},
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
        signature: ClassSignature<'src>,
        fields: ClassFields,
        method_bodies: MethodBodyRegistry,
        cons_bodies: ConsBodyRegistry,
    ) -> Self {
        let methods = signature.methods.take_registry().combine(method_bodies, |data, body| {
            data.combine(body, |data, body| MethodModel::new(data.1, body))
        });
        let constructors =
            signature.constructors.combine(cons_bodies, |data, body| ConsModel::new(data.1, body));
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
        with_fields: ClassWithFieldRegistry<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        with_fields
            .transform_with_self(
                |registry, cls_id, cls_data| {
                    let cons_bodies = cls_data
                        .class_signature()
                        .constructors
                        .iter()
                        .map(|(cons_id, cons_data)| {
                            (
                                cons_id,
                                ConsBody::resolve(registry, cls_id, cons_id, &cons_data.0, errors),
                            )
                        })
                        .collect();

                    let method_bodies = cls_data
                        .class_signature()
                        .methods
                        .iter()
                        .map(|(method_name_id, methods)| {
                            (
                                method_name_id,
                                methods
                                    .iter()
                                    .map(|(method_overload_id, method_data)| {
                                        (
                                            method_overload_id,
                                            MethodBody::resolve(
                                                registry,
                                                cls_id,
                                                (method_name_id, method_overload_id).into(),
                                                &method_data.0,
                                                errors,
                                            ),
                                        )
                                    })
                                    .collect(),
                            )
                        })
                        .collect();
                    (method_bodies, cons_bodies)
                },
                |(_def, signature, fields), (method_bodies, cons_bodies)| {
                    ClassModel::new(signature, fields, method_bodies, cons_bodies)
                },
            )
            .take_registry()
    }
}
