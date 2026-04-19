use std::rc::Rc;

use derive_more::Display;

use crate::{
    analyzer::{
        AnalysisError, GeneralError,
        def::{ClassDefsRegistry, ClassMemberNamesCtx, ClassNamesCtx, GetMemberNamesCtx},
        expr::PrimitiveExpr,
        registry::{
            ClassId, ClassRegistry, ConsId, FieldId, FieldRegistry, FieldRegistryBuilder,
            LibClassId,
        },
        signature::{ClassSignature, ClassSignatureCtx, GetClsSignatureCtx},
    },
    ast::{ShipCallExpr, ShipCallableExprAll, ShipExprAll, ShipId, ShipPrimaryAll, ShipVarDef},
    diagnostics::Renderable,
    parser::WithParserLoc,
    stdlib::StdlibCtx,
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

pub trait FieldResolutionCtx<'src>: StdlibCtx + ClassSignatureCtx + ClassNamesCtx<'src> {}
impl<'src, Ctx: StdlibCtx + ClassSignatureCtx + ClassNamesCtx<'src>> FieldResolutionCtx<'src>
    for Ctx
{
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
        ctx: &impl FieldResolutionCtx<'src>,
        signature: &ClassSignature,
        call: &Rc<ShipCallExpr<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> FieldModel {
        match &call.expr {
            ShipCallableExprAll::Cons(cls_name) => {
                //TODO lib classes!!!
                match ctx.cls_names().get_class(cls_name) {
                    Ok(cls_id) => {
                        let own_cls_id = signature.id;
                        if ctx.is_cls_subcls_of(cls_id, own_cls_id).0 {
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
                            FieldModel { field_type: cls_id, init_expr: FieldExpr::Invalid }
                        } else {
                            let arg_types: Vec<_> =
                                args.iter().map(|model| model.field_type).collect();

                            ctx.get_cls_signature(&cls_id)
                                .constructors
                                .find_matching_cons(ctx, &arg_types, &call.args)
                                .map(|(cons_id, _data)| FieldModel {
                                    field_type: cls_id,
                                    init_expr: FieldExpr::Cons {
                                        class: cls_id,
                                        cons: cons_id,
                                        args,
                                    },
                                })
                                .unwrap_or_else(|e| Self::invalid(errors, cls_id, e))
                        }
                    },
                    Err(_) => Self::invalid(
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
        ctx: &impl FieldResolutionCtx<'src>,
        signature: &ClassSignature,
        def: &Rc<ShipVarDef<'src>>,
        errors: &mut Vec<AnalysisError<'src>>,
    ) -> Self {
        match &def.expr {
            ShipExprAll::Call(call) => Self::resolve_call(ctx, signature, call, errors),
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

pub type ClassFieldsRegistry = ClassRegistry<ClassFields>;

pub trait ClassFieldsCtx {
    fn cls_fields(&self) -> &ClassFieldsRegistry;
}
impl ClassFieldsCtx for ClassFieldsRegistry {
    fn cls_fields(&self) -> &ClassFieldsRegistry {
        self
    }
}

pub trait GetClsFieldsCtx: StdlibCtx + ClassFieldsCtx {
    fn get_cls_fields(&self, cls_id: &ClassId) -> &ClassFields;
}
impl<Ctx: StdlibCtx + ClassFieldsCtx> GetClsFieldsCtx for Ctx {
    fn get_cls_fields(&self, cls_id: &ClassId) -> &ClassFields {
        match cls_id {
            ClassId::User(user_class_id) => self.cls_fields().get(user_class_id),
            ClassId::Lib(lib_class_id) => self.stdlib().cls_fields(lib_class_id),
            ClassId::Invalid => self.stdlib().invalid_fields(),
        }
    }
}

pub trait FindFieldCtx<'src>:
    StdlibCtx + ClassSignatureCtx + ClassMemberNamesCtx<'src> + ClassFieldsCtx
{
    fn find_field(
        &self,
        cls_id: ClassId,
        field_name: &Rc<ShipId<'src>>,
    ) -> Result<(FieldId, &FieldModel), FieldError<'src>>;
}
impl<'src, Ctx: StdlibCtx + ClassSignatureCtx + ClassMemberNamesCtx<'src> + ClassFieldsCtx>
    FindFieldCtx<'src> for Ctx
{
    fn find_field(
        &self,
        cls_id: ClassId,
        field_name: &Rc<ShipId<'src>>,
    ) -> Result<(FieldId, &FieldModel), FieldError<'src>> {
        let signature = self.get_cls_signature(&cls_id);
        let members = self.get_member_names(&cls_id);
        let fields = self.get_cls_fields(&cls_id);

        if let Some(field_id) = members.fields.get_by_name(field_name.id) {
            Ok((field_id, fields.registry.get(&field_id)))
        } else if cls_id == LibClassId::Class.into() || cls_id == ClassId::Invalid {
            Err(FieldError::UndefinedFieldName { name: field_name.clone() })
        } else {
            self.find_field(signature.parent, field_name)
        }
    }
}

pub fn init_class_fields_registry<'src>(
    ctx: &impl FieldResolutionCtx<'src>,
    defs: &ClassDefsRegistry<'src>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> ClassFieldsRegistry {
    defs.iter()
        .map(|(cls_id, def)| {
            (
                cls_id,
                ClassFields {
                    registry: def
                        .fields
                        .iter()
                        .map(|(field_id, field)| {
                            (
                                field_id,
                                FieldModel::resolve(
                                    ctx,
                                    ctx.signatures().get(&cls_id),
                                    field,
                                    errors,
                                ),
                            )
                        })
                        .collect(),
                },
            )
        })
        .collect()
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
    fn render(&self, _src: &impl crate::ByteSource<'src>) -> String {
        match self {
            FieldError::InvalidInitExpr { expr: _ } => {
                format!("Only constructors & primitive allowed in field initializers")
            },
            FieldError::RecursiveInitExpr { call: _ } => {
                format!("Recursive field initializer expressiondetected")
            },
            FieldError::UndefinedFieldName { name } => {
                format!("Field with name `{}` was not found", name.id)
            },
        }
    }
}

impl<'src> WithParserLoc for FieldError<'src> {
    fn loc(&self) -> crate::parser::ParserLoc {
        match self {
            FieldError::InvalidInitExpr { expr } => expr.loc(),
            FieldError::RecursiveInitExpr { call } => call.loc(),
            FieldError::UndefinedFieldName { name } => name.loc(),
        }
    }
}
