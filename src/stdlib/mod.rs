mod model;

use std::collections::HashMap;

use crate::{
    analyzer::{
        def::ClassMemberNames,
        field::ClassFields,
        registry::{
            ClassId, LibClassId, NameRegistry, NameRegistryBuilder, Registry, RegistryBuilder,
        },
        signature::{ClassSignature, MethodSignature, ParamsSignature},
    },
    stdlib::model::{LibClassModel, models},
};

pub struct ShipStdLib {
    inner: HashMap<LibClassId, (ClassSignature, ClassFields, ClassMemberNames<'static>)>,
    invalid_cls: ClassSignature,
    invalid_fields: ClassFields,
    invalid_member_names: ClassMemberNames<'static>,
}

impl ShipStdLib {
    pub fn cls_signature(&self, cls_id: &LibClassId) -> &ClassSignature {
        &self.inner.get(cls_id).unwrap().0
    }
    pub fn cls_fields(&self, cls_id: &LibClassId) -> &ClassFields {
        &self.inner.get(cls_id).unwrap().1
    }
    pub fn cls_member_names(&self, cls_id: &LibClassId) -> &ClassMemberNames<'static> {
        &self.inner.get(cls_id).unwrap().2
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

fn process_model(model: LibClassModel) -> (ClassSignature, ClassFields, ClassMemberNames<'static>) {
    let mut cons_builder = RegistryBuilder::default();
    for cons in model.constructors {
        cons_builder.insert(cons);
    }

    let mut method_builder = NameRegistryBuilder::default();
    for (name, overlaods) in model.methods {
        let mut overload_builder = RegistryBuilder::default();
        for overload in overlaods {
            overload_builder.insert(overload);
        }
        let _ = method_builder.insert(name, overload_builder.build());
    }
    let (method_names, methods) = method_builder.build();

    let mut field_builder = NameRegistryBuilder::default();
    for (name, field) in model.fields {
        let _ = field_builder.insert(name, field);
    }
    let (field_names, fields) = field_builder.build();

    (
        ClassSignature {
            id: model.id.into(),
            parent: model.parent.into(),
            constructors: cons_builder.build(),
            methods,
        },
        ClassFields { registry: fields },
        ClassMemberNames { methods: method_names, fields: field_names },
    )
}

pub fn stdlib() -> ShipStdLib {
    let inner = models().into_values().map(|model| (model.id, process_model(model))).collect();

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
