use std::{collections::HashMap, iter::Enumerate, marker::PhantomData};

pub trait RegistryIdProvider {
    type Id: RegistryId;
    fn get(&mut self) -> Self::Id;
}

pub struct SimpleIdProvider<Id: RegistryId> {
    last_id: usize,
    phantom: PhantomData<Id>,
}
impl<Id: RegistryId> RegistryIdProvider for SimpleIdProvider<Id> {
    type Id = Id;

    #[inline]
    fn get(&mut self) -> Self::Id {
        let res = self.last_id;
        self.last_id += 1;
        Id::from_index(res)
    }
}
impl<Id: RegistryId> Default for SimpleIdProvider<Id> {
    #[inline]
    fn default() -> Self {
        Self { last_id: 0, phantom: Default::default() }
    }
}

#[allow(private_bounds)] //by design
pub trait RegistryId: InnerRegistryId + HasProvider {}
impl<I: InnerRegistryId + HasProvider> RegistryId for I {}

trait InnerRegistryId: Copy + Eq {
    fn as_index(&self) -> usize;
    fn from_index(idx: usize) -> Self;
}
pub trait HasProvider {
    type Provider: RegistryIdProvider<Id = Self>;
}

pub struct RegistryBuilder<Id: RegistryId, V> {
    registry: Registry<Id, V>,
    id_provider: <Id as HasProvider>::Provider,
}
impl<Id: RegistryId, V> RegistryBuilder<Id, V> {
    #[inline]
    pub fn new(id_provider: <Id as HasProvider>::Provider) -> Self {
        Self { registry: Default::default(), id_provider }
    }
    #[inline]
    pub fn insert(&mut self, value: V) -> Id {
        let id = self.id_provider.get();
        self.registry.insert(id, value);
        id
    }

    #[inline]
    pub fn curr(&self) -> &Registry<Id, V> {
        &self.registry
    }

    #[inline]
    pub fn build(self) -> Registry<Id, V> {
        self.registry
    }
}
impl<Id: RegistryId, V> Default for RegistryBuilder<Id, V>
where
    <Id as HasProvider>::Provider: Default,
{
    fn default() -> Self {
        Self { registry: Default::default(), id_provider: Default::default() }
    }
}

pub struct Registry<Id: RegistryId, V> {
    inner: Vec<Option<V>>,
    phantom: PhantomData<Id>,
}
impl<Id: RegistryId, V> Registry<Id, V> {
    #[inline]
    fn new_with(len: usize) -> Self {
        let mut inner = Vec::with_capacity(len);
        inner.resize_with(len, || Option::None);
        Self { inner, phantom: PhantomData }
    }
    #[inline]
    fn insert(&mut self, id: Id, value: V) {
        let idx = id.as_index();
        if idx >= self.inner.len() {
            self.inner.resize_with(idx + 1, || Option::None);
        }
        self.inner[idx] = Some(value);
    }
    #[inline]
    fn get_mut(&mut self, id: &Id) -> &mut V {
        self.inner.get_mut(id.as_index()).and_then(Option::as_mut).unwrap()
    }
    #[inline]
    fn replace<F>(&mut self, id: &Id, f: F)
    where
        F: FnOnce(Option<V>) -> V,
    {
        let idx = id.as_index();
        let mut temp = Option::None;
        std::mem::swap(&mut temp, &mut self.inner[idx]);
        self.inner[idx] = Some(f(temp));
    }

    #[inline]
    pub fn get(&self, id: &Id) -> &V {
        self.inner[id.as_index()].as_ref().unwrap()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Id, V> {
        self.into_iter()
    }

    #[inline]
    pub fn transform<V2, F>(self, mut f: F) -> Registry<Id, V2>
    where
        F: FnMut(Id, V) -> V2,
    {
        //TODO optimise
        let mut new_registry = Registry::new_with(self.inner.len());
        for (id, old_value) in self.into_iter() {
            new_registry.insert(id, f(id, old_value));
        }
        new_registry
    }

    #[inline]
    pub fn combine<V2, V3, F>(self, other: Registry<Id, V2>, f: F) -> Registry<Id, V3>
    where
        F: Fn(V, V2) -> V3,
    {
        let t = self
            .inner
            .into_iter()
            .zip(other.inner)
            .map(|(v, v2)| match (v, v2) {
                (Some(a), Some(b)) => Some(f(a, b)),
                _ => None,
            })
            .collect();
        Registry { inner: t, phantom: PhantomData }
    }
}

impl<Id: RegistryId, V> Default for Registry<Id, V> {
    #[inline]
    fn default() -> Self {
        Self { inner: Default::default(), phantom: Default::default() }
    }
}

pub struct Iter<'a, Id: RegistryId, V: 'a> {
    inner: Enumerate<std::slice::Iter<'a, Option<V>>>,
    phantom: PhantomData<Id>,
}
impl<'a, Id: RegistryId, V: 'a> Iterator for Iter<'a, Id, V> {
    type Item = (Id, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((idx, value)) => {
                    if let Some(v) = value {
                        return Some((Id::from_index(idx), v));
                    }
                },
                None => return None,
            }
        }
    }
}

pub struct IntoIter<Id: RegistryId, V> {
    inner: Enumerate<std::vec::IntoIter<Option<V>>>,
    phantom: PhantomData<Id>,
}
impl<Id: RegistryId, V> Iterator for IntoIter<Id, V> {
    type Item = (Id, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((idx, value)) => {
                    if let Some(v) = value {
                        return Some((Id::from_index(idx), v));
                    }
                },
                None => return None,
            }
        }
    }
}

impl<'a, Id: RegistryId, V> IntoIterator for &'a Registry<Id, V> {
    type Item = (Id, &'a V);

    type IntoIter = Iter<'a, Id, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter { inner: self.inner.iter().enumerate(), phantom: PhantomData }
    }
}

impl<Id: RegistryId, V> IntoIterator for Registry<Id, V> {
    type Item = (Id, V);

    type IntoIter = IntoIter<Id, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self.inner.into_iter().enumerate(), phantom: PhantomData }
    }
}

impl<Id: RegistryId, V> FromIterator<(Id, V)> for Registry<Id, V> {
    fn from_iter<T: IntoIterator<Item = (Id, V)>>(iter: T) -> Self {
        let mut registry = Self::default();
        for (id, value) in iter {
            registry.insert(id, value);
        }
        registry
    }
}

pub struct NameRegistryBuilder<'key, Id: RegistryId, V> {
    registry: NameRegistry<'key, Id, V>,
    id_provider: <Id as HasProvider>::Provider,
}
impl<'key, Id: RegistryId, V> NameRegistryBuilder<'key, Id, V> {
    #[inline]
    pub fn new(id_provider: <Id as HasProvider>::Provider) -> Self {
        Self { registry: Default::default(), id_provider }
    }
    #[inline]
    pub fn insert(&mut self, name: &'key str, value: V) -> Option<(&V, V)> {
        match self.registry.get_by_name(name) {
            Some(id) => Some((self.registry.get(&id), value)),
            None => {
                let id = self.id_provider.get();
                self.registry.insert(name, id, value);
                None
            },
        }
    }
    #[inline]
    pub fn update<F>(&mut self, name: &'key str, update: F) -> Id
    where
        F: FnOnce(Option<V>) -> V,
    {
        match self.registry.get_by_name(name) {
            Some(id) => {
                self.registry.replace(&id, update);
                id
            },
            None => {
                let value = update(None);
                let id = self.id_provider.get();
                self.registry.insert(name, id, value);
                id
            },
        }
    }

    pub fn curr(&self) -> &NameRegistry<'key, Id, V> {
        &self.registry
    }

    #[inline]
    pub fn build(self) -> NameRegistry<'key, Id, V> {
        self.registry
    }
}
impl<'key, Id: RegistryId, V> Default for NameRegistryBuilder<'key, Id, V>
where
    <Id as HasProvider>::Provider: Default,
{
    fn default() -> Self {
        Self { registry: Default::default(), id_provider: Default::default() }
    }
}

pub struct NameRegistry<'key, Id: RegistryId, V: 'key> {
    name_to_id: HashMap<&'key str, Id>,
    id_to_value: Registry<Id, V>,
}

impl<'key, Id: RegistryId, V: 'key> NameRegistry<'key, Id, V> {
    #[inline]
    fn insert(&mut self, name: &'key str, id: Id, value: V) {
        self.name_to_id.insert(name, id);
        self.id_to_value.insert(id, value);
    }
    #[inline]
    fn get_mut(&mut self, id: &Id) -> &mut V {
        self.id_to_value.get_mut(id)
    }
    #[inline]
    fn replace<F>(&mut self, id: &Id, f: F)
    where
        F: FnOnce(Option<V>) -> V,
    {
        self.id_to_value.replace(id, f);
    }

    #[inline]
    pub fn get_by_name(&self, name: &str) -> Option<Id> {
        self.name_to_id.get(name).copied()
    }

    #[inline]
    pub fn get(&self, id: &Id) -> &V {
        self.id_to_value.get(id)
    }

    #[inline]
    pub fn get_name(&self, id: &Id) -> &'key str {
        self.name_to_id.iter().find(|(_name, name_id)| id == *name_id).unwrap().0
    }
    #[inline]
    pub fn iter(&self) -> Iter<'_, Id, V> {
        self.id_to_value.iter()
    }

    pub fn registry(&self) -> &Registry<Id, V> {
        &self.id_to_value
    }

    pub fn take_registry(self) -> Registry<Id, V> {
        self.id_to_value
    }

    pub fn transform<V2, Fp>(self, f: Fp) -> NameRegistry<'key, Id, V2>
    where
        Fp: FnMut(Id, V) -> V2,
    {
        NameRegistry { name_to_id: self.name_to_id, id_to_value: self.id_to_value.transform(f) }
    }

    pub fn transform_with_self<V2, V3, Fp, Fc>(
        self,
        mut process: Fp,
        combine: Fc,
    ) -> NameRegistry<'key, Id, V3>
    where
        Fp: FnMut(&Self, Id, &V) -> V2,
        Fc: Fn(V, V2) -> V3,
    {
        let mut new_registry = Registry::new_with(self.id_to_value.inner.len());
        for (id, old_value) in self.id_to_value.iter() {
            new_registry.insert(id, process(&self, id, old_value));
        }

        NameRegistry {
            name_to_id: self.name_to_id,
            id_to_value: self.id_to_value.combine(new_registry, combine),
        }
    }
}

impl<'key, Id: RegistryId, V: 'key> Default for NameRegistry<'key, Id, V> {
    fn default() -> Self {
        Self { name_to_id: Default::default(), id_to_value: Default::default() }
    }
}

mod class {
    use super::{
        HasProvider, InnerRegistryId, NameRegistry, NameRegistryBuilder, Registry, RegistryBuilder,
        SimpleIdProvider,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum LibClassId {
        Class,
        AnyValue,
        AnyRef,
        Integer,
        Real,
        Boolean,
        Array,
        List,
        String,
        Char,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ClassId {
        User(UserClassId),
        Lib(LibClassId),
        Invalid,
    }

    impl From<UserClassId> for ClassId {
        fn from(value: UserClassId) -> Self {
            Self::User(value)
        }
    }
    impl From<LibClassId> for ClassId {
        fn from(value: LibClassId) -> Self {
            Self::Lib(value)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserClassId(u32);
    impl UserClassId {}

    impl InnerRegistryId for UserClassId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u32)
        }
    }
    impl HasProvider for UserClassId {
        type Provider = SimpleIdProvider<UserClassId>;
    }

    pub type ClassNameRegistry<'key, V> = NameRegistry<'key, UserClassId, V>;
    pub type ClassNameRegistryBuilder<'key, V> = NameRegistryBuilder<'key, UserClassId, V>;
    pub type ClassRegistry<V> = Registry<UserClassId, V>;
    pub type ClassRegistryBuilder<V> = RegistryBuilder<UserClassId, V>;
}
pub use class::*;

mod method {
    use crate::analyzer::registry::NameRegistryBuilder;

    use super::{
        HasProvider, InnerRegistryId, NameRegistry, Registry, RegistryBuilder, SimpleIdProvider,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MethodOverloadId(u16);
    impl InnerRegistryId for MethodOverloadId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u16)
        }
    }
    impl HasProvider for MethodOverloadId {
        type Provider = SimpleIdProvider<MethodOverloadId>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MethodNameId(u16);
    impl InnerRegistryId for MethodNameId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u16)
        }
    }
    impl HasProvider for MethodNameId {
        type Provider = SimpleIdProvider<MethodNameId>;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MethodId(MethodNameId, MethodOverloadId);
    impl From<(MethodNameId, MethodOverloadId)> for MethodId {
        fn from(value: (MethodNameId, MethodOverloadId)) -> Self {
            Self(value.0, value.1)
        }
    }

    pub type MethodNameRegistry<'key, V> =
        NameRegistry<'key, MethodNameId, Registry<MethodOverloadId, V>>;
    pub type MethodNameRegistryBuilder<'key, V> =
        NameRegistryBuilder<'key, MethodNameId, RegistryBuilder<MethodOverloadId, V>>;

    pub type MethodRegistry<V> = Registry<MethodNameId, Registry<MethodOverloadId, V>>;
    pub type MethodRegistryBuilder<V> =
        RegistryBuilder<MethodNameId, RegistryBuilder<MethodOverloadId, V>>;
    impl<V> MethodRegistry<V> {
        pub fn get_method(&self, id: &MethodId) -> &V {
            self.get(&id.0).get(&id.1)
        }
    }
}
pub use method::*;

mod constructor {
    use super::{HasProvider, InnerRegistryId, Registry, RegistryBuilder, SimpleIdProvider};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ConsId(u32);
    impl InnerRegistryId for ConsId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u32)
        }
    }
    impl HasProvider for ConsId {
        type Provider = SimpleIdProvider<ConsId>;
    }
    pub type ConsRegistry<V> = Registry<ConsId, V>;
    pub type ConsRegistryBuilder<V> = RegistryBuilder<ConsId, V>;
}
pub use constructor::*;

mod field {
    use super::{
        HasProvider, InnerRegistryId, NameRegistry, NameRegistryBuilder, Registry, RegistryBuilder,
        SimpleIdProvider,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FieldId(u32);
    impl InnerRegistryId for FieldId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u32)
        }
    }
    impl HasProvider for FieldId {
        type Provider = SimpleIdProvider<FieldId>;
    }
    pub type FieldNameRegistry<'key, V> = NameRegistry<'key, FieldId, V>;
    pub type FieldNameRegistryBuilder<'key, V> = NameRegistryBuilder<'key, FieldId, V>;
    pub type FieldRegistry<V> = Registry<FieldId, V>;
    pub type FieldRegistryBuilder<V> = RegistryBuilder<FieldId, V>;
}
pub use field::*;

mod variable {
    use super::{
        HasProvider, InnerRegistryId, NameRegistry, NameRegistryBuilder, Registry, RegistryBuilder,
        SimpleIdProvider,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VarId(u32);
    impl InnerRegistryId for VarId {
        #[inline]
        fn as_index(&self) -> usize {
            self.0 as usize
        }

        #[inline]
        fn from_index(idx: usize) -> Self {
            Self(idx as u32)
        }
    }
    impl HasProvider for VarId {
        type Provider = SimpleIdProvider<VarId>;
    }
    pub type VarNameRegistry<'key, V> = NameRegistry<'key, VarId, V>;
    pub type VarNameRegistryBuilder<'key, V> = NameRegistryBuilder<'key, VarId, V>;
    pub type VarRegistry<V> = Registry<VarId, V>;
    pub type VarRegistryBuilder<V> = RegistryBuilder<VarId, V>;
}
pub use variable::*;
