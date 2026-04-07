use super::registry::*;
use std::collections::HashMap;
use std::rc::Rc;

use derive_more::Display;

use crate::analyzer::field::FieldError;
use crate::ast::{
    ShipClassDef, ShipClassMemberAll, ShipConsDef, ShipId, ShipMethodDef, ShipParams, ShipVarDef,
};
use crate::diagnostics::Renderable;

pub type ClassDefRegistry<'src> = ClassRegistry<'src, Rc<ShipClassDef<'src>>>;

impl<'src> ClassDefRegistry<'src> {
    pub fn new(defs: &Vec<Rc<ShipClassDef<'src>>>, errors: &mut Vec<ClassError<'src>>) -> Self {
        let mut builder = ClassRegistryBuilder::default();

        for cls in defs {
            if let Some((old, new)) = builder.insert(cls.class_id.id, cls.clone()) {
                errors.push(ClassError::DuplicateClassName { fst: old.clone(), snd: new });
            }
        }
        builder.build()
    }
}

pub type ParamsSignature = Vec<ClassId>;
pub struct MethodSignature {
    pub params: ParamsSignature,
    pub return_type: Option<ClassId>,
}

pub type MethodSignatureRegistry<'src> =
    MethodRegistry<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;
pub type MethodSignatureRegistryBuilder<'src> =
    MethodRegistryBuilder<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;

pub type ConsSignatureRegistry<'src> = ConsRegistry<(Rc<ShipConsDef<'src>>, ParamsSignature)>;
pub type ConsSignatureRegistryBuilder<'src> =
    ConsRegistryBuilder<(Rc<ShipConsDef<'src>>, ParamsSignature)>;

pub type FieldNameRegistry<'src> = FieldRegistry<'src, Rc<ShipVarDef<'src>>>;
pub type FieldNameRegistryBuilder<'src> = FieldRegistryBuilder<'src, Rc<ShipVarDef<'src>>>;

pub struct ClassSignature<'src> {
    pub id: UserClassId,
    pub parent: ClassId,
    pub constructors: ConsSignatureRegistry<'src>,
    pub methods: MethodSignatureRegistry<'src>,
    pub fields: FieldNameRegistry<'src>,
}
pub type ClassSignatureRegistry<'src> =
    NameRegistry<'src, UserClassId, (Rc<ShipClassDef<'src>>, ClassSignature<'src>)>;

impl<'src> ClassSignatureRegistry<'src> {
    fn get_user_class(
        defs: &ClassDefRegistry<'src>,
        cls_name: &Rc<ShipId<'src>>,
    ) -> Result<UserClassId, ClassError<'src>> {
        match defs.get_by_name(cls_name.id) {
            Some(cls_id) => Ok(cls_id),
            None => Err(ClassError::UndefinedClass { cls_name: cls_name.clone() }),
        }
    }

    fn get_class(
        defs: &ClassDefRegistry<'src>,
        cls_name: &Rc<ShipId<'src>>,
        errors: &mut Vec<ClassError<'src>>,
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
        errors: &mut Vec<ClassError<'src>>,
    ) -> ParamsSignature {
        params_node
            .params
            .iter()
            .map(|param| {
                let cls_name = &param.var_type;
                Self::get_class(defs, cls_name, errors)
            })
            .collect()
    }

    fn resolve_method(
        defs: &ClassDefRegistry<'src>,
        method_node: &Rc<ShipMethodDef<'src>>,
        errors: &mut Vec<ClassError<'src>>,
    ) -> MethodSignature {
        let params = Self::resolve_params(defs, &method_node.params, errors);
        let return_type = method_node
            .return_type
            .as_ref()
            .map(|cls_name| Self::get_class(defs, cls_name, errors));
        MethodSignature { params, return_type }
    }

    pub fn new(defs: ClassDefRegistry<'src>, errors: &mut Vec<ClassError<'src>>) -> Self {
        defs.transform_with_self(
            |defs, id, def| {
                let parent = def
                    .parent_id
                    .as_ref()
                    .map(|cls_name| Self::get_class(defs, cls_name, errors))
                    .unwrap_or(ClassId::Lib(LibClassId::AnyRef));

                let mut constructors = ConsSignatureRegistryBuilder::default();
                let mut methods = MethodSignatureRegistryBuilder::default();
                let mut fields = FieldNameRegistryBuilder::default();

                for member in &def.members {
                    match member {
                        ShipClassMemberAll::VarDef(node) => {
                            if let Some((old, new)) = fields.insert(node.var_id.id, node.clone()) {
                                errors.push(ClassError::DuplicateField {
                                    fst: old.clone(),
                                    snd: new,
                                });
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
                    id,
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
            Some(VisitStatus::Fresh) => Err(CircularInheritance { chain: vec![data.0.clone()] }),
            None => {
                visited.insert(id, VisitStatus::Fresh);
                let res = match &data.1.parent {
                    ClassId::User(parent_id) => {
                        self.check_parent(*parent_id, visited).map_err(|mut e| {
                            e.chain.push(data.0.clone());
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

    pub fn check_inheritance(self, errors: &mut Vec<ClassError<'src>>) -> Self {
        let mut visited: HashMap<UserClassId, VisitStatus> = HashMap::new();
        self.transform_with_self(
            |registry, id, _data| match registry.check_parent(id, &mut visited) {
                Ok(is_valid) => is_valid,
                Err(e) => {
                    errors.push(ClassError::CircularInheritance(e));
                    false
                },
            },
            |data, is_valid| {
                (
                    data.0,
                    if is_valid {
                        data.1
                    } else {
                        ClassSignature { parent: ClassId::Invalid, ..data.1 }
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
    #[display("undefined class with name `{cls_name}`")]
    UndefinedClass { cls_name: Rc<ShipId<'src>> },
    #[display("circular inheritance chain")]
    CircularInheritance(CircularInheritance<'src>),
    #[display("duplicate class name")]
    DuplicateClassName { fst: Rc<ShipClassDef<'src>>, snd: Rc<ShipClassDef<'src>> },
    #[display("duplicate constructor def")]
    DuplicateConstructor { fst: Rc<ShipConsDef<'src>>, snd: Rc<ShipConsDef<'src>> },
    #[display("duplicate field")]
    DuplicateField { fst: Rc<ShipVarDef<'src>>, snd: Rc<ShipVarDef<'src>> },
    #[display("{_0}")]
    Field(FieldError<'src>),
}

impl<'src> Renderable<'src> for ClassError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
