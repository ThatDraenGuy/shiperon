use crate::{
    analyzer::{
        AnalysisError, GeneralError,
        registry::{
            ClassId, ClassNameRegistry, ClassNameRegistryBuilder, ClassRegistry, ConsRegistry,
            ConsRegistryBuilder, FieldNameRegistry, FieldNameRegistryBuilder, FieldRegistry,
            LibClassId, MethodNameRegistry, MethodNameRegistryBuilder, MethodRegistry,
            RegistryBuilder, UserClassId,
        },
        signature::ClassError,
        stdlib::ShipStdLib,
    },
    ast::{ShipClassDef, ShipClassMemberAll, ShipConsDef, ShipId, ShipMethodDef, ShipVarDef},
};
use std::rc::Rc;

pub struct ClassMemberNames<'src> {
    pub methods: MethodNameRegistry<'src>,
    pub fields: FieldNameRegistry<'src>,
}

pub type ClassMemberNamesRegistry<'src> = ClassRegistry<ClassMemberNames<'src>>;

impl<'src> ClassMemberNamesRegistry<'src> {
    pub fn get_member_names<'a>(
        &'a self,
        stdlib: &'a ShipStdLib,
        cls_id: &ClassId,
    ) -> &'a ClassMemberNames {
        todo!()
    }
}

pub type ClassDefsRegistry<'src> = ClassRegistry<ClassDef<'src>>;

pub struct ClassDef<'src> {
    pub node: Rc<ShipClassDef<'src>>,
    pub constructors: ConsRegistry<Rc<ShipConsDef<'src>>>,
    pub methods: MethodRegistry<Rc<ShipMethodDef<'src>>>,
    pub fields: FieldRegistry<Rc<ShipVarDef<'src>>>,
}

fn init_class_members_registry<'src>(
    def: &Rc<ShipClassDef<'src>>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> (ClassMemberNames<'src>, ClassDef<'src>) {
    let mut methods_builder = MethodNameRegistryBuilder::default();
    let mut fields_builder = FieldNameRegistryBuilder::default();
    let mut cons_builder = ConsRegistryBuilder::default();

    for member in &def.members {
        match member {
            ShipClassMemberAll::VarDef(field) => {
                if let Some((old, new)) = fields_builder.insert(field.var_id.id, field.clone()) {
                    errors.push(ClassError::DuplicateField { fst: old.clone(), snd: new }.into());
                }
            },
            ShipClassMemberAll::MethodDef(method) => {
                methods_builder.update(method.method_id.id, |maybe_old| match maybe_old {
                    Some(mut old) => {
                        //TODO check same params?
                        old.insert(method.clone());
                        old
                    },
                    None => {
                        let mut builder = RegistryBuilder::default();
                        builder.insert(method.clone());
                        builder
                    },
                });
            },
            ShipClassMemberAll::ConsDef(cons) => {
                cons_builder.insert(cons.clone());
            },
        }
    }
    let (field_names, field_defs) = fields_builder.build();
    let cons_defs = cons_builder.build();
    let (method_names, method_builders) = methods_builder.build();
    let method_defs = method_builders.transform(|id, builder| builder.build());

    (
        ClassMemberNames { methods: method_names, fields: field_names },
        ClassDef {
            node: def.clone(),
            constructors: cons_defs,
            methods: method_defs,
            fields: field_defs,
        },
    )
}

pub fn init_cls_registry<'src>(
    defs: &Vec<Rc<ShipClassDef<'src>>>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> (ClassNameRegistry<'src>, ClassMemberNamesRegistry<'src>, ClassDefsRegistry<'src>) {
    let mut builder = ClassNameRegistryBuilder::default();
    for cls in defs {
        if let Some((old, new)) =
            builder.insert(cls.class_id.id, init_class_members_registry(cls, errors))
        {
            errors.push(
                ClassError::DuplicateClassName { fst: old.1.node.clone(), snd: new.1.node.clone() }
                    .into(),
            );
        }
    }
    let (names, data) = builder.build();
    let (member_names, defs) = data.split();
    (names, member_names, defs)
}

impl<'src> ClassNameRegistry<'src> {
    pub fn get_user_class(
        &self,
        cls_name: &Rc<ShipId<'src>>,
    ) -> Result<UserClassId, AnalysisError<'src>> {
        match self.get_by_name(cls_name.id) {
            Some(cls_id) => Ok(cls_id),
            None => Err(GeneralError::UndefinedClass { cls_name: cls_name.clone() }.into()),
        }
    }

    pub fn get_class(&self, cls_name: &Rc<ShipId<'src>>) -> Result<ClassId, AnalysisError<'src>> {
        Ok(match cls_name.id {
            "Class" => LibClassId::Class.into(),
            "AnyRef" => LibClassId::AnyRef.into(),
            "AnyValue" => LibClassId::AnyValue.into(),
            "Integer" => LibClassId::Integer.into(),
            "Real" => LibClassId::Real.into(),
            "Boolean" => LibClassId::Boolean.into(),
            "Char" => LibClassId::Char.into(),
            "String" => LibClassId::String.into(),
            "Array" => LibClassId::Array.into(),
            _ => match Self::get_user_class(self, cls_name) {
                Ok(user_class) => ClassId::User(user_class),
                Err(e) => return Err(e),
            },
        })
    }

    pub fn get_class_with_err(
        &self,
        cls_name: &Rc<ShipId<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> ClassId {
        match self.get_class(cls_name) {
            Ok(cls) => cls,
            Err(e) => {
                errors.push(e);
                ClassId::Invalid
            },
        }
    }
}
