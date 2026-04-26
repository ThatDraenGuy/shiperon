mod model;

use std::collections::HashMap;

use inkwell::values::{AnyValueEnum, BasicValueEnum, FunctionValue};

use crate::{
    analyzer::{
        def::ClassMemberNames,
        field::ClassFields,
        registry::{
            ClassId, LibClassId, MethodRegistry, NameRegistry, NameRegistryBuilder, Registry,
            RegistryBuilder,
        },
        signature::ClassSignature,
    },
    codegen::{CodegenContext, ShipLLVMContext},
    stdlib::model::{LibClassModel, models},
};

// pub struct LibConsModel {
//     call_impl: for<'ctx, 'src> fn(&'ctx CodegenContext<'ctx, 'src>) -> BasicValueEnum<'ctx>,
//     def_impl: for<'ctx, 'src> fn(&'ctx CodegenContext<'ctx, 'src>),
// }

pub struct LibMethodImpl {
    pub call_impl: for<'ctx, 'src> fn(
        ctx: &'ctx CodegenContext<'ctx, 'src>,
        object: BasicValueEnum<'ctx>,
        args: &[BasicValueEnum<'ctx>],
    ) -> AnyValueEnum<'ctx>,
    pub def_impl: for<'ctx> fn(&ShipLLVMContext<'ctx>) -> Option<FunctionValue<'ctx>>,
}

pub struct LibClassImpl {
    pub methods: MethodRegistry<LibMethodImpl>,
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
    let mut cons_builder = RegistryBuilder::default();
    for cons in model.constructors {
        cons_builder.insert(cons);
    }

    let mut method_builder = NameRegistryBuilder::default();
    for (name, overlaods) in model.methods {
        let mut overload_builder = RegistryBuilder::default();
        for lib_model in overlaods {
            overload_builder.insert((lib_model.signature, lib_model.method_impl));
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
            constructors: cons_builder.build(),
            methods: method_signatures,
        },
        ClassFields { registry: fields },
        ClassMemberNames { methods: method_names, fields: field_names },
        LibClassImpl { methods: method_impls },
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
