// use std::cell::LazyCell;

// use once_cell::unsync::OnceCell;

// use crate::{
//     CompilerConfig, Lexer, Parser,
//     analyzer::{field::ClassWithFieldRegistry, registry::LibClassId, signature::ClassSignature},
// };

use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    CompilerConfig, Lexer, Parser,
    analyzer::{
        field::{ClassFields, ClassWithFieldRegistry, WithClassFields},
        registry::{LibClassId, NameRegistry, Registry, RegistryBuilder, RegistryId},
        signature::{ClassDefRegistry, ClassSignature, ClassSignatureRegistry, WithClassSignature},
        stages::Stage2,
    },
};

const STD_SRC: &[u8; 3555] = include_bytes!("std.po");

pub struct StdLibRegistry {
    inner: HashMap<LibClassId, Stage2<'static>>,
    invalid_cls: ClassSignature<'static>,
    invalid_fields: ClassFields,
}
impl StdLibRegistry {
    pub fn cls_signature(&self, cls_id: &LibClassId) -> &ClassSignature<'static> {
        self.inner.get(cls_id).unwrap().class_signature()
    }
    pub fn cls_fields(&self, cls_id: &LibClassId) -> &ClassFields {
        self.inner.get(cls_id).unwrap().class_fields()
    }
    pub fn invalid_signature(&self) -> &ClassSignature<'static> {
        &self.invalid_cls
    }
    pub fn invalid_fields(&self) -> &ClassFields {
        &self.invalid_fields
    }
}

pub fn stdlib() -> StdLibRegistry {
    let parser =
        Parser::new(Lexer::of_str(str::from_utf8(STD_SRC).unwrap()), CompilerConfig::internal());
    let parse_data = parser.consume_parse();

    let ast = parse_data.program.unwrap();
    let mut errors = Vec::new();

    let fake_lib = StdLibRegistry {
        inner: HashMap::new(),
        invalid_cls: ClassSignature::invalid(),
        invalid_fields: ClassFields { registry: Registry::empty() },
    };
    let class_defs = ClassDefRegistry::new(&ast.classes, &mut errors);
    let class_signatures = ClassSignatureRegistry::new(class_defs, &mut errors);
    let checked_signatures = class_signatures.check_inheritance(&mut errors);
    let with_fields = ClassWithFieldRegistry::new(
        WithStd { lib: Rc::new(fake_lib), user: checked_signatures, phantom: PhantomData },
        &mut errors,
    );

    let lib_reg = with_fields.map(|with_fields, lib| {
        with_fields.transform_with_self(
            |registry, user_id, data| match registry.get_name(&user_id) {
                "Class" => LibClassId::Class,
                "AnyRef" => LibClassId::AnyRef,
                "AnyValue" => LibClassId::AnyValue,
                "Integer" => LibClassId::Integer,
                "Real" => LibClassId::Real,
                "Boolean" => LibClassId::Boolean,
                "Char" => LibClassId::Char,
                "String" => LibClassId::String,
                "Array" => LibClassId::Array,
                "List" => LibClassId::List,
                _ => unreachable!("non existent lib class"),
            },
            |data, lib_id| (lib_id, data),
        )
    });

    StdLibRegistry {
        inner: lib_reg.unwrap().take_registry().into_iter().map(|tup| tup.1).collect(),
        invalid_cls: ClassSignature::invalid(),
        invalid_fields: ClassFields { registry: Registry::empty() },
    }
}

pub struct WithStd<'std, T> {
    pub lib: Rc<StdLibRegistry>,
    user: T,
    phantom: PhantomData<&'std str>,
}
impl<'std, T> Deref for WithStd<'std, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}
impl<'std, T> DerefMut for WithStd<'std, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.user
    }
}
impl<'std, T> WithStd<'std, T> {
    pub fn unwrap(self) -> T {
        self.user
    }
    pub fn wrap(lib: Rc<StdLibRegistry>, user: T) -> Self {
        Self { lib, user, phantom: PhantomData }
    }
    pub fn map<V, F>(self, f: F) -> WithStd<'std, V>
    where
        F: FnOnce(T, Rc<StdLibRegistry>) -> V,
    {
        let lib = self.lib;
        let new_user = f(self.user, lib.clone());
        WithStd { lib, user: new_user, phantom: PhantomData }
    }
}

impl<'std, 'key, Id: RegistryId, V> WithStd<'std, &NameRegistry<'key, Id, V>> {
    pub fn registry(&self) -> WithStd<'std, &Registry<Id, V>> {
        WithStd { lib: self.lib.clone(), user: &self.user.registry(), phantom: PhantomData }
    }
}
