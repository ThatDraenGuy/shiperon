mod model;

use std::collections::HashMap;

use derive_more::{From, Unwrap};
use inkwell::values::FunctionValue;

use crate::{
    analyzer::{
        def::ClassMemberNames,
        field::ClassFields,
        registry::{
            ClassId, ConsRegistry, LibClassId, MethodRegistry, NameRegistry, NameRegistryBuilder,
            Registry, RegistryBuilder,
        },
        signature::ClassSignature,
    },
    codegen::LLVMCtx,
    stdlib::model::{
        LibClassModel, LibConsObjectImpl, LibConsValueImpl, LibMethodObjectImpl,
        LibMethodValueImpl, models,
    },
};

pub struct LibObjectImpl {
    pub init_impl: for<'ctx> fn(&dyn LLVMCtx<'ctx>) -> FunctionValue<'ctx>,
    pub constructors: ConsRegistry<LibConsObjectImpl>,
    pub methods: MethodRegistry<LibMethodObjectImpl>,
}

pub struct LibValueImpl {
    pub constructors: ConsRegistry<LibConsValueImpl>,
    pub methods: MethodRegistry<LibMethodValueImpl>,
}

#[derive(From, Unwrap)]
#[unwrap(ref)]
pub enum LibClassImpl {
    Object(LibObjectImpl),
    Value(LibValueImpl),
    Blanket,
}

pub struct ShipStdLib {
    inner:
        HashMap<LibClassId, (ClassSignature, ClassFields, ClassMemberNames<'static>, LibClassImpl)>,
    invalid_cls: ClassSignature,
    invalid_fields: ClassFields,
    invalid_member_names: ClassMemberNames<'static>,
}

impl ShipStdLib {
    pub fn cls_name(&self, cls_id: &LibClassId) -> &'static str {
        match cls_id {
            LibClassId::Class => "Class",
            LibClassId::AnyValue => "AnyValue",
            LibClassId::AnyRef => "AnyRef",
            LibClassId::Integer => "Integer",
            LibClassId::Real => "Real",
            LibClassId::Boolean => "Boolean",
            LibClassId::Array => "Array",
            LibClassId::List => "List",
            LibClassId::String => "String",
            LibClassId::Char => "Char",
        }
    }
    pub fn cls_signature(&self, cls_id: &LibClassId) -> &ClassSignature {
        &self.inner.get(cls_id).unwrap().0
    }
    pub fn cls_fields(&self, cls_id: &LibClassId) -> &ClassFields {
        &self.inner.get(cls_id).unwrap().1
    }
    pub fn cls_member_names(&self, cls_id: &LibClassId) -> &ClassMemberNames<'static> {
        &self.inner.get(cls_id).unwrap().2
    }
    pub fn cls_impl(&self, cls_id: &LibClassId) -> &LibClassImpl {
        &self.inner.get(cls_id).unwrap().3
    }
    pub fn invalid_signature(&self) -> &ClassSignature {
        &self.invalid_cls
    }
    pub fn invalid_fields(&self) -> &ClassFields {
        &self.invalid_fields
    }
    pub fn invalid_member_names(&self) -> &ClassMemberNames<'static> {
        &self.invalid_member_names
    }
}

pub trait StdlibCtx {
    fn stdlib(&self) -> &ShipStdLib;
}
impl StdlibCtx for ShipStdLib {
    fn stdlib(&self) -> &ShipStdLib {
        self
    }
}

fn process_model(
    model: LibClassModel,
) -> (ClassSignature, ClassFields, ClassMemberNames<'static>, LibClassImpl) {
    match model {
        LibClassModel::Object(model) => {
            let mut cons_builder = RegistryBuilder::default();
            for cons in model.constructors {
                cons_builder.insert(cons);
            }
            let (cons_signatures, cons_impls) = cons_builder.build().split();

            let mut method_builder = NameRegistryBuilder::default();
            for (name, overlaods) in model.methods {
                let mut overload_builder = RegistryBuilder::default();
                for lib_model in overlaods {
                    overload_builder.insert(lib_model);
                }
                let _ = method_builder.insert(name, overload_builder.build().split());
            }
            let (method_names, methods) = method_builder.build();
            let (method_signatures, method_impls) = methods.split();

            let mut field_builder = NameRegistryBuilder::default();
            for (name, field) in model.fields {
                let _ = field_builder.insert(name, field);
            }
            let (field_names, fields) = field_builder.build();

            (
                ClassSignature {
                    id: model.id.into(),
                    parent: model.parent.into(),
                    constructors: cons_signatures,
                    methods: method_signatures,
                },
                ClassFields { registry: fields },
                ClassMemberNames { methods: method_names, fields: field_names },
                LibObjectImpl {
                    init_impl: model.init_impl,
                    methods: method_impls,
                    constructors: cons_impls,
                }
                .into(),
            )
        },
        LibClassModel::Value(model) => {
            let mut cons_builder = RegistryBuilder::default();
            for cons in model.constructors {
                cons_builder.insert(cons);
            }
            let (cons_signatures, cons_impls) = cons_builder.build().split();

            let mut method_builder = NameRegistryBuilder::default();
            for (name, overlaods) in model.methods {
                let mut overload_builder = RegistryBuilder::default();
                for lib_model in overlaods {
                    overload_builder.insert(lib_model);
                }
                let _ = method_builder.insert(name, overload_builder.build().split());
            }
            let (method_names, methods) = method_builder.build();
            let (method_signatures, method_impls) = methods.split();

            let mut field_builder = NameRegistryBuilder::default();
            for (name, field) in model.fields {
                let _ = field_builder.insert(name, field);
            }
            let (field_names, fields) = field_builder.build();

            (
                ClassSignature {
                    id: model.id.into(),
                    parent: model.parent.into(),
                    constructors: cons_signatures,
                    methods: method_signatures,
                },
                ClassFields { registry: fields },
                ClassMemberNames { methods: method_names, fields: field_names },
                LibValueImpl { methods: method_impls, constructors: cons_impls }.into(),
            )
        },
        LibClassModel::Blanket(model) => {
            let mut cons_builder = RegistryBuilder::default();
            for cons in model.constructors {
                cons_builder.insert(cons);
            }
            let cons_signatures = cons_builder.build();

            let mut method_builder = NameRegistryBuilder::default();
            for (name, overlaods) in model.methods {
                let mut overload_builder = RegistryBuilder::default();
                for lib_model in overlaods {
                    overload_builder.insert(lib_model);
                }
                let _ = method_builder.insert(name, overload_builder.build());
            }
            let (method_names, methods) = method_builder.build();
            let method_signatures = methods;

            let mut field_builder = NameRegistryBuilder::default();
            for (name, field) in model.fields {
                let _ = field_builder.insert(name, field);
            }
            let (field_names, fields) = field_builder.build();

            (
                ClassSignature {
                    id: model.id.into(),
                    parent: model.parent.into(),
                    constructors: cons_signatures,
                    methods: method_signatures,
                },
                ClassFields { registry: fields },
                ClassMemberNames { methods: method_names, fields: field_names },
                LibClassImpl::Blanket,
            )
        },
    }
}

pub fn stdlib() -> ShipStdLib {
    let inner = models().into_values().map(|model| (model.id(), process_model(model))).collect();

    ShipStdLib {
        inner,
        invalid_cls: ClassSignature {
            id: ClassId::Invalid,
            parent: ClassId::Invalid,
            constructors: Registry::empty(),
            methods: Registry::empty(),
        },
        invalid_fields: ClassFields { registry: Registry::empty() },
        invalid_member_names: ClassMemberNames {
            methods: NameRegistry::empty(),
            fields: NameRegistry::empty(),
        },
    }
}
