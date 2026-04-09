use super::registry::*;
use std::collections::HashMap;
use std::rc::Rc;

use derive_more::Display;

use crate::analyzer::stages::Stage1;
use crate::analyzer::{AnalysisError, GeneralError};
use crate::ast::{
    ShipArgs, ShipClassDef, ShipClassMemberAll, ShipConsDef, ShipId, ShipMethodDef, ShipParams,
    ShipVarDef,
};
use crate::diagnostics::Renderable;

pub type ClassDefRegistry<'src> = ClassNameRegistry<'src, Rc<ShipClassDef<'src>>>;

impl<'src> ClassDefRegistry<'src> {
    pub fn new(defs: &Vec<Rc<ShipClassDef<'src>>>, errors: &mut Vec<AnalysisError<'src>>) -> Self {
        let mut builder = ClassNameRegistryBuilder::default();

        for cls in defs {
            if let Some((old, new)) = builder.insert(cls.class_id.id, cls.clone()) {
                errors.push(ClassError::DuplicateClassName { fst: old.clone(), snd: new }.into());
            }
        }
        builder.build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsSignature {
    param_types: Vec<ClassId>,
}
impl ParamsSignature {
    pub fn new(param_types: Vec<ClassId>) -> Self {
        Self { param_types }
    }

    pub fn matches<'src, V>(&self, arg_types: &[ClassId], registry: &ClassRegistry<V>) -> (bool, u8)
    where
        V: WithClassSignature<'src>,
    {
        if arg_types.len() != self.param_types.len() {
            (false, 0)
        } else {
            self.param_types
                .iter()
                .zip(arg_types)
                .map(|(param, arg)| registry.is_cls_subcls_of(*arg, *param))
                .reduce(|(valid, degree), (is_subcls, diff)| match (valid, is_subcls) {
                    (true, true) => (true, degree + diff),
                    (true, false) => (false, 0),
                    (false, _) => (false, degree),
                })
                .unwrap_or((true, 0)) //zero-arg constructor
        }
    }
}

pub trait WithParamsSignature {
    fn params_signature(&self) -> &ParamsSignature;
}
impl WithParamsSignature for ParamsSignature {
    #[inline]
    fn params_signature(&self) -> &ParamsSignature {
        self
    }
}
impl<T> WithParamsSignature for (T, ParamsSignature) {
    fn params_signature(&self) -> &ParamsSignature {
        &self.1
    }
}

impl<V: WithParamsSignature> ConsRegistry<V> {
    pub fn find_matching_cons<'src, C: WithClassSignature<'src>>(
        &self,
        param_types: &[ClassId],
        registry: &ClassRegistry<C>,
        node: &Rc<ShipArgs<'src>>,
    ) -> Result<(ConsId, &V), ConsError<'src>> {
        self.iter()
            .filter_map(|(cons_id, data)| {
                let (is_match, degree) = data.params_signature().matches(param_types, registry);
                if is_match { Some((cons_id, degree)) } else { None }
            })
            .min_by(|fst, snd| fst.1.cmp(&snd.1))
            .map(|(cons_id, _degree)| (cons_id, self.get(&cons_id)))
            .ok_or(ConsError::NoMatchingCons { args: node.clone() })
    }
}

#[derive(Debug, Clone, Display)]
pub enum ConsError<'src> {
    #[display("no matching cons found")]
    NoMatchingCons { args: Rc<ShipArgs<'src>> },
}
impl<'src> Renderable<'src> for ConsError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}

pub struct MethodSignature {
    pub params: ParamsSignature,
    pub return_type: Option<ClassId>,
}

pub trait WithMethodSignature {
    fn method_signature(&self) -> &MethodSignature;
}
impl WithMethodSignature for MethodSignature {
    #[inline]
    fn method_signature(&self) -> &MethodSignature {
        self
    }
}
impl<T> WithMethodSignature for (T, MethodSignature) {
    #[inline]
    fn method_signature(&self) -> &MethodSignature {
        &self.1
    }
}

impl<'src, C: WithClassSignature<'src>> ClassRegistry<C> {
    pub fn get_cls(&self, cls_id: &ClassId) -> &C {
        match cls_id {
            ClassId::User(user_class_id) => self.get(&user_class_id),
            ClassId::Lib(lib_class_id) => todo!(),
            ClassId::Invalid => todo!(),
        }
    }
    pub fn find_matching_method(
        &self,
        cls_id: ClassId,
        name: &'src str,
        param_types: &[ClassId],
        name_node: &Rc<ShipId<'src>>,
        args_node: &Rc<ShipArgs<'src>>,
    ) -> Result<(ClassId, MethodId), MethodError<'src>> {
        match cls_id {
            ClassId::User(user_class_id) => {
                let signature = self.get(&user_class_id).class_signature();
                let methods = &signature.methods;
                let name_id = methods
                    .get_by_name(name)
                    .ok_or(MethodError::UndefinedMethod { name: name_node.clone() })?;
                methods
                    .get(&name_id)
                    .iter()
                    .filter_map(|(overload_id, data)| {
                        let (is_match, degree) =
                            data.method_signature().params.matches(param_types, self);
                        if is_match { Some((overload_id, degree)) } else { None }
                    })
                    .min_by(|fst, snd| fst.1.cmp(&snd.1))
                    .map(|(overload_id, _degree)| Ok((cls_id, (name_id, overload_id).into())))
                    .unwrap_or_else(|| {
                        let parent = signature.parent;
                        if parent == LibClassId::Class.into() {
                            Err(MethodError::NoOverload { args: args_node.clone() })
                        } else {
                            self.find_matching_method(
                                parent,
                                name,
                                param_types,
                                name_node,
                                args_node,
                            )
                        }
                    })
            },
            ClassId::Lib(lib_class_id) => todo!(),
            ClassId::Invalid => todo!(),
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum MethodError<'src> {
    #[display("no method name found")]
    UndefinedMethod { name: Rc<ShipId<'src>> },
    #[display("no overload matching call")]
    NoOverload { args: Rc<ShipArgs<'src>> },
}
impl<'src> Renderable<'src> for MethodError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}

pub type MethodSignatureRegistry<'src> =
    MethodNameRegistry<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;
pub type MethodSignatureRegistryBuilder<'src> =
    MethodNameRegistryBuilder<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;

pub type ConsSignatureRegistry<'src> = ConsRegistry<(Rc<ShipConsDef<'src>>, ParamsSignature)>;
pub type ConsSignatureRegistryBuilder<'src> =
    ConsRegistryBuilder<(Rc<ShipConsDef<'src>>, ParamsSignature)>;

pub type FieldNamesRegistry<'src> = FieldNameRegistry<'src, Rc<ShipVarDef<'src>>>;
pub type FieldNamesRegistryBuilder<'src> = FieldNameRegistryBuilder<'src, Rc<ShipVarDef<'src>>>;

pub struct ClassSignature<'src> {
    pub id: ClassId,
    pub parent: ClassId,
    pub constructors: ConsSignatureRegistry<'src>,
    pub methods: MethodSignatureRegistry<'src>,
    pub fields: FieldNamesRegistry<'src>,
}

pub trait WithClassDef<'src> {
    fn class_def(&self) -> &Rc<ShipClassDef<'src>>;
}
impl<'src> WithClassDef<'src> for Rc<ShipClassDef<'src>> {
    #[inline]
    fn class_def(&self) -> &Rc<ShipClassDef<'src>> {
        self
    }
}

pub trait WithClassSignature<'src> {
    fn class_signature(&self) -> &ClassSignature<'src>;
}
impl<'src> WithClassSignature<'src> for ClassSignature<'src> {
    #[inline]
    fn class_signature(&self) -> &ClassSignature<'src> {
        self
    }
}

impl<'src, V: WithClassSignature<'src>> ClassRegistry<V> {
    pub fn is_cls_subcls_of<C: Into<ClassId>, P: Into<ClassId>>(
        &self,
        child: C,
        parent: P,
    ) -> (bool, u8) {
        let parent = parent.into();
        let mut current = child.into();
        let mut diff = 0;
        loop {
            if current == parent {
                return (true, diff);
            }
            if current == LibClassId::Class.into() {
                return (false, diff);
            }
            let signature = match &current {
                ClassId::User(user_class_id) => self.get(user_class_id).class_signature(),
                ClassId::Lib(lib_class_id) => todo!(),
                ClassId::Invalid => {
                    return (false, diff);
                },
            };
            current = signature.parent;
            diff += 1;
        }
    }
}

pub type ClassSignatureRegistry<'src> = NameRegistry<'src, UserClassId, Stage1<'src>>;

impl<'src> ClassSignatureRegistry<'src> {
    fn get_user_class(
        defs: &ClassDefRegistry<'src>,
        cls_name: &Rc<ShipId<'src>>,
    ) -> Result<UserClassId, AnalysisError<'src>> {
        match defs.get_by_name(cls_name.id) {
            Some(cls_id) => Ok(cls_id),
            None => Err(GeneralError::UndefinedClass { cls_name: cls_name.clone() }.into()),
        }
    }

    fn get_class(
        defs: &ClassDefRegistry<'src>,
        cls_name: &Rc<ShipId<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> ClassId {
        //TODO: lib classes
        match Self::get_user_class(defs, cls_name) {
            Ok(user_class) => ClassId::User(user_class),
            Err(e) => {
                errors.push(e);
                ClassId::Invalid
            },
        }
    }

    fn resolve_params(
        defs: &ClassDefRegistry<'src>,
        params_node: &Rc<ShipParams<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> ParamsSignature {
        ParamsSignature::new(
            params_node
                .params
                .iter()
                .map(|param| {
                    let cls_name = &param.var_type;
                    Self::get_class(defs, cls_name, errors)
                })
                .collect(),
        )
    }

    fn resolve_method(
        defs: &ClassDefRegistry<'src>,
        method_node: &Rc<ShipMethodDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> MethodSignature {
        let params = Self::resolve_params(defs, &method_node.params, errors);
        let return_type = method_node
            .return_type
            .as_ref()
            .map(|cls_name| Self::get_class(defs, cls_name, errors));
        MethodSignature { params, return_type }
    }

    pub fn new(defs: ClassDefRegistry<'src>, errors: &mut Vec<AnalysisError<'src>>) -> Self {
        defs.transform_with_self(
            |defs, id, def| {
                let parent = def
                    .parent_id
                    .as_ref()
                    .map(|cls_name| Self::get_class(defs, cls_name, errors))
                    .unwrap_or(ClassId::Lib(LibClassId::AnyRef));

                let mut constructors = ConsSignatureRegistryBuilder::default();
                let mut methods = MethodSignatureRegistryBuilder::default();
                let mut fields = FieldNamesRegistryBuilder::default();

                for member in &def.members {
                    match member {
                        ShipClassMemberAll::VarDef(node) => {
                            if let Some((old, new)) = fields.insert(node.var_id.id, node.clone()) {
                                errors.push(
                                    ClassError::DuplicateField { fst: old.clone(), snd: new }
                                        .into(),
                                );
                            }
                        },
                        ShipClassMemberAll::MethodDef(node) => {
                            let signature = Self::resolve_method(defs, node, errors);
                            methods.update(node.method_id.id, |maybe_old| match maybe_old {
                                Some(mut old) => {
                                    //TODO check same params?
                                    old.insert((node.clone(), signature));
                                    old
                                },
                                None => {
                                    let mut builder = RegistryBuilder::default();
                                    builder.insert((node.clone(), signature));
                                    builder
                                },
                            });
                        },
                        ShipClassMemberAll::ConsDef(node) => {
                            let params = Self::resolve_params(defs, &node.params, errors);
                            constructors.insert((node.clone(), params));
                        },
                    }
                }

                ClassSignature {
                    id: id.into(),
                    parent,
                    constructors: constructors.build(),
                    methods: methods.build().transform(|_id, builder| builder.build()),
                    fields: fields.build(),
                }
            },
            |def, signature| (def, signature),
        )
    }

    fn check_parent(
        &self,
        id: UserClassId,
        visited: &mut HashMap<UserClassId, VisitStatus>,
    ) -> Result<bool, CircularInheritance<'src>> {
        let data = self.get(&id);
        match visited.get(&id) {
            Some(VisitStatus::Valid) => Ok(true),
            Some(VisitStatus::Invalid) => Ok(false),
            Some(VisitStatus::Fresh) => {
                Err(CircularInheritance { chain: vec![data.class_def().clone()] })
            },
            None => {
                visited.insert(id, VisitStatus::Fresh);
                let res = match &data.class_signature().parent {
                    ClassId::User(parent_id) => {
                        self.check_parent(*parent_id, visited).map_err(|mut e| {
                            e.chain.push(data.class_def().clone());
                            CircularInheritance { chain: e.chain }
                        })
                    },
                    ClassId::Lib(_lib_class_id) => Ok(true),
                    ClassId::Invalid => Ok(true),
                };
                visited.insert(
                    id,
                    if res.is_ok() { VisitStatus::Valid } else { VisitStatus::Invalid },
                );
                res
            },
        }
    }

    pub fn check_inheritance(self, errors: &mut Vec<AnalysisError<'src>>) -> Self {
        let mut visited: HashMap<UserClassId, VisitStatus> = HashMap::new();
        self.transform_with_self(
            |registry, id, _data| match registry.check_parent(id, &mut visited) {
                Ok(is_valid) => is_valid,
                Err(e) => {
                    errors.push(ClassError::CircularInheritance(e).into());
                    false
                },
            },
            |(def, signature), is_valid| {
                (
                    def,
                    if is_valid {
                        signature
                    } else {
                        ClassSignature { parent: ClassId::Invalid, ..signature }
                    },
                )
            },
        )
    }
}

enum VisitStatus {
    Fresh,
    Valid,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct CircularInheritance<'src> {
    chain: Vec<Rc<ShipClassDef<'src>>>,
}

#[derive(Debug, Clone, Display)]
pub enum ClassError<'src> {
    #[display("circular inheritance chain")]
    CircularInheritance(CircularInheritance<'src>),
    #[display("duplicate class name")]
    DuplicateClassName { fst: Rc<ShipClassDef<'src>>, snd: Rc<ShipClassDef<'src>> },
    #[display("duplicate constructor def")]
    DuplicateConstructor { fst: Rc<ShipConsDef<'src>>, snd: Rc<ShipConsDef<'src>> },
    #[display("duplicate field")]
    DuplicateField { fst: Rc<ShipVarDef<'src>>, snd: Rc<ShipVarDef<'src>> },
}

impl<'src> Renderable<'src> for ClassError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
