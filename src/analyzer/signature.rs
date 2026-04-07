use super::registry::*;
use std::rc::Rc;

use derive_more::Display;
use std::collections::HashMap;

use crate::ast::{
    ShipClassDef, ShipClassMemberAll, ShipConsDef, ShipId, ShipMethodDef, ShipParams,
};
use crate::diagnostics::Renderable;

pub type ClassDefRegistry<'src> = ClassRegistry<'src, Rc<ShipClassDef<'src>>>;

impl<'src> ClassDefRegistry<'src> {
    pub fn new(defs: &Vec<Rc<ShipClassDef<'src>>>, errors: &mut Vec<ClassError<'src>>) -> Self {
        let mut builder = ClassRegistryBuilder::default();

        for cls in defs {
            builder.insert_or_update(
                cls.class_id.id,
                || cls.clone(),
                |old| {
                    errors.push(ClassError::DuplicateClassName {
                        fst: old.clone(),
                        snd: cls.clone(),
                    }); //TODO think - skipping class?
                },
            );
        }
        builder.build()
    }
}

pub type ParamsSignature = Vec<ClassId>;
pub struct MethodSignature {
    params: ParamsSignature,
    return_type: Option<ClassId>,
}

pub type MethodSignatureRegistry<'src> =
    MethodRegistry<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;
pub type MethodSignatureRegistryBuilder<'src> =
    MethodRegistryBuilder<'src, (Rc<ShipMethodDef<'src>>, MethodSignature)>;

pub type ConsSignatureRegistry<'src> = ConsRegistry<(Rc<ShipConsDef<'src>>, ParamsSignature)>;
pub type ConsSignatureRegistryBuilder<'src> =
    ConsRegistryBuilder<(Rc<ShipConsDef<'src>>, ParamsSignature)>;

pub struct ClassSignature<'src> {
    pub id: UserClassId,
    pub parent: ClassId,
    pub constructors: ConsSignatureRegistry<'src>,
    pub methods: MethodSignatureRegistry<'src>,
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
        // let mut signatures = HashMap::new();
        defs.transform_with_self(|defs, id, def| {
            let parent = def
                .parent_id
                .as_ref()
                .map(|cls_name| Self::get_class(defs, cls_name, errors))
                .unwrap_or(ClassId::Lib(LibClassId::AnyRef));

            // let mut methods = HashMap::new();
            // let mut constructors: HashMap<ParamsSignature, Rc<ShipConsDef<'src>>> = HashMap::new();
            let mut constructors = ConsSignatureRegistryBuilder::default();
            let mut methods = MethodSignatureRegistryBuilder::default();

            for member in &def.members {
                match member {
                    ShipClassMemberAll::VarDef(node) => todo!(),
                    ShipClassMemberAll::MethodDef(node) => {
                        let signature = Self::resolve_method(defs, node, errors);
                        methods.insert_or_update(
                            node.method_id.id,
                            || todo!(),
                            |mut builder| todo!(),
                        );
                    },
                    ShipClassMemberAll::ConsDef(node) => {
                        let params = Self::resolve_params(defs, &node.params, errors);
                        constructors.insert((node.clone(), params));
                    },
                }
            }
            (
                def.clone(),
                ClassSignature { id, parent, constructors: constructors.build(), methods: todo!() },
            )
        })
    }
}

#[derive(Debug, Clone, Display)]
pub enum ClassError<'src> {
    #[display("undefined class with name `{cls_name}`")]
    UndefinedClass { cls_name: Rc<ShipId<'src>> },
    #[display("circular inheritance chain")]
    CircularInheritance { chain: Vec<Rc<ShipClassDef<'src>>> },
    #[display("duplicate class name")]
    DuplicateClassName { fst: Rc<ShipClassDef<'src>>, snd: Rc<ShipClassDef<'src>> },
    #[display("duplicate constructor def")]
    DuplicateConstructor { fst: Rc<ShipConsDef<'src>>, snd: Rc<ShipConsDef<'src>> },
}

impl<'src> Renderable<'src> for ClassError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
