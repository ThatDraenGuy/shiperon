use std::rc::Rc;

use derive_more::Display;

use crate::{
    analyzer::{
        AnalysisError, GeneralError,
        expr::PrimitiveExpr,
        registry::{
            ClassId, ClassNameRegistry, ClassRegistry, ConsId, FieldId, FieldRegistry,
            FieldRegistryBuilder, LibClassId,
        },
        signature::{ClassSignature, ClassSignatureRegistry, WithClassSignature},
        stages::Stage2,
    },
    ast::{
        ShipCallExpr, ShipCallableExprAll, ShipClassDef, ShipExprAll, ShipId, ShipPrimaryAll,
        ShipVarDef,
    },
    diagnostics::Renderable,
};

pub enum FieldExpr {
    Primitive(PrimitiveExpr),
    Cons { class: ClassId, cons: ConsId, args: Vec<FieldModel> },
    Invalid,
}
impl From<PrimitiveExpr> for FieldExpr {
    fn from(value: PrimitiveExpr) -> Self {
        Self::Primitive(value)
    }
}

pub struct FieldModel {
    pub field_type: ClassId,
    pub init_expr: FieldExpr,
}

impl FieldModel {
    #[inline]
    fn invalid<'src, C: Into<ClassId>, E: Into<AnalysisError<'src>>>(
        errors: &mut Vec<AnalysisError<'src>>,
        cls_id: C,
        e: E,
    ) -> Self {
        errors.push(e.into());
        Self { field_type: cls_id.into(), init_expr: FieldExpr::Invalid }
    }

    fn resolve_primary<'src>(
        primary: &ShipPrimaryAll<'src>,
    ) -> Result<FieldModel, FieldError<'src>> {
        match primary {
            ShipPrimaryAll::Int(int_node) => Ok(FieldModel {
                field_type: LibClassId::Integer.into(),
                init_expr: PrimitiveExpr::Integer(int_node.int).into(),
            }),
            ShipPrimaryAll::Float(float_node) => Ok(FieldModel {
                field_type: LibClassId::Real.into(),
                init_expr: PrimitiveExpr::Real(float_node.float).into(),
            }),
            ShipPrimaryAll::String(string_node) => Ok(FieldModel {
                field_type: LibClassId::String.into(),
                init_expr: PrimitiveExpr::String(string_node.string.clone()).into(),
            }),
            ShipPrimaryAll::Char(char_node) => Ok(FieldModel {
                field_type: LibClassId::Char.into(),
                init_expr: PrimitiveExpr::Char(char_node.char).into(),
            }),
            primary => {
                Err(FieldError::InvalidInitExpr { expr: ShipExprAll::Primary(primary.clone()) })
            },
        }
    }

    fn resolve_primitive<'src>(expr: &ShipExprAll<'src>) -> Result<FieldModel, FieldError<'src>> {
        match expr {
            ShipExprAll::Primary(primary) => Self::resolve_primary(primary),
            _ => Err(FieldError::InvalidInitExpr { expr: expr.clone() }),
        }
    }

    fn resolve_call<'src>(
        registry: &ClassSignatureRegistry<'src>,
        signature: &ClassSignature<'src>,
        call: &Rc<ShipCallExpr<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> FieldModel {
        match &call.expr {
            ShipCallableExprAll::Cons(cls_name) => {
                //TODO lib classes!!!
                match registry.get_by_name(cls_name.id) {
                    Some(cls_id) => {
                        let own_cls_id = signature.id;
                        if registry.registry().is_cls_subcls_of(cls_id, own_cls_id).0 {
                            return Self::invalid(
                                errors,
                                ClassId::Invalid,
                                FieldError::RecursiveInitExpr { call: call.clone() },
                            );
                        }

                        let mut has_errors = false;
                        let args: Vec<_> = call
                            .args
                            .exprs
                            .iter()
                            .map(Self::resolve_primitive)
                            .map(|res| match res {
                                Ok(model) => model,
                                Err(e) => {
                                    has_errors = true;
                                    Self::invalid(errors, ClassId::Invalid, e)
                                },
                            })
                            .collect();
                        if has_errors {
                            FieldModel { field_type: cls_id.into(), init_expr: FieldExpr::Invalid }
                        } else {
                            let arg_types: Vec<_> =
                                args.iter().map(|model| model.field_type).collect();

                            registry
                                .get(&cls_id)
                                .class_signature()
                                .constructors
                                .find_matching_cons(&arg_types, registry.registry(), &call.args)
                                .map(|(cons_id, _data)| FieldModel {
                                    field_type: cls_id.into(),
                                    init_expr: FieldExpr::Cons {
                                        class: cls_id.into(),
                                        cons: cons_id,
                                        args,
                                    },
                                })
                                .unwrap_or_else(|e| Self::invalid(errors, cls_id, e))
                        }
                    },
                    None => Self::invalid(
                        errors,
                        ClassId::Invalid,
                        GeneralError::UndefinedClass { cls_name: cls_name.clone() },
                    ),
                }
            },
            _ => Self::invalid(
                errors,
                ClassId::Invalid,
                FieldError::InvalidInitExpr { expr: ShipExprAll::Call(call.clone()) },
            ),
        }
    }

    pub fn resolve<'src>(
        registry: &ClassSignatureRegistry<'src>,
        signature: &ClassSignature<'src>,
        id: FieldId,
        def: &Rc<ShipVarDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        match &def.expr {
            ShipExprAll::Call(call) => Self::resolve_call(registry, signature, call, errors),
            ShipExprAll::Primary(primary) => Self::resolve_primary(primary)
                .unwrap_or_else(|e| Self::invalid(errors, ClassId::Invalid, e)),
            expr => Self::invalid(
                errors,
                ClassId::Invalid,
                FieldError::InvalidInitExpr { expr: expr.clone() },
            ),
        }
    }
}

pub type FieldModelRegistry = FieldRegistry<FieldModel>;
pub type FieldModelRegistryBuilder = FieldRegistryBuilder<FieldModel>;

pub struct ClassFields {
    pub registry: FieldModelRegistry,
}
pub trait WithClassFields {
    fn class_fields(&self) -> &ClassFields;
}
impl WithClassFields for ClassFields {
    #[inline]
    fn class_fields(&self) -> &ClassFields {
        self
    }
}
impl<'src, V: WithClassFields + WithClassSignature<'src>> ClassRegistry<V> {
    pub fn find_field(
        &self,
        cls_id: ClassId,
        field_name: &Rc<ShipId<'src>>,
    ) -> Result<(FieldId, &FieldModel), FieldError<'src>> {
        let cls = self.get_cls(&cls_id);
        let signature = cls.class_signature();
        let fields = cls.class_fields();
        if let Some(field_id) = signature.fields.get_by_name(field_name.id) {
            Ok((field_id, fields.registry.get(&field_id)))
        } else if cls_id == LibClassId::Class.into() {
            Err(FieldError::UndefinedFieldName { name: field_name.clone() })
        } else {
            self.find_field(signature.parent, field_name)
        }
    }
}

pub type ClassWithFieldRegistry<'src> = ClassNameRegistry<'src, Stage2<'src>>;

impl<'src> ClassWithFieldRegistry<'src> {
    pub fn new(
        signatures: ClassSignatureRegistry<'src>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        signatures.transform_with_self(
            |signatures, _cls_id, data| ClassFields {
                registry: data
                    .class_signature()
                    .fields
                    .iter()
                    .map(|(field_id, def)| {
                        (
                            field_id,
                            FieldModel::resolve(
                                signatures,
                                data.class_signature(),
                                field_id,
                                def,
                                errors,
                            ),
                        )
                    })
                    .collect(),
            },
            |data, fields| (data.0, data.1, fields),
        )
    }
}

#[derive(Debug, Clone, Display)]
pub enum FieldError<'src> {
    #[display("invalid expr in field init `{expr}`")]
    InvalidInitExpr { expr: ShipExprAll<'src> },
    #[display("field init expr invokes owning class cons")]
    RecursiveInitExpr { call: Rc<ShipCallExpr<'src>> },
    #[display("undefined field")]
    UndefinedFieldName { name: Rc<ShipId<'src>> },
}

impl<'src> Renderable<'src> for FieldError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        todo!()
    }
}
