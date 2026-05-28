use super::registry::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use derive_more::Display;
use itertools::Itertools;

use crate::analyzer::AnalysisError;
use crate::analyzer::def::{
    ClassDefsRegistry, ClassMemberNamesCtx, ClassMemberNamesRegistry, ClassNamesCtx,
    GetMemberNamesCtx,
};
use crate::ast::{
    ShipArgs, ShipClassDef, ShipConsDef, ShipId, ShipMethodDef, ShipParams, ShipVarDef,
};
use crate::diagnostics::Renderable;
use crate::parser::{ParserLoc, WithParserLoc};
use crate::{ShipStdLib, StdlibCtx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsSignature {
    pub param_types: Vec<ClassId>,
}
impl ParamsSignature {
    pub fn new(param_types: Vec<ClassId>) -> Self {
        Self { param_types }
    }
    pub fn empty() -> Self {
        Self { param_types: vec![] }
    }

    pub fn matches(&self, ctx: &impl GetClsSignatureCtx, arg_types: &[ClassId]) -> (bool, u8) {
        if arg_types.len() != self.param_types.len() {
            (false, 0)
        } else {
            self.param_types
                .iter()
                .zip(arg_types)
                .map(|(param, arg)| ctx.is_cls_subcls_of(*arg, *param))
                .reduce(|(valid, degree), (is_subcls, diff)| match (valid, is_subcls) {
                    (true, true) => (true, degree + diff),
                    (true, false) => (false, 0),
                    (false, _) => (false, degree),
                })
                .unwrap_or((true, 0)) //zero-arg constructor
        }
    }

    pub fn annotate_types<V, I: Iterator<Item = V>>(&self, args: I) -> Vec<(V, ClassId)> {
        args.zip(self.param_types.iter().copied()).collect()
    }
}

impl ConsSignatureRegistry {
    pub fn find_matching_cons<'src>(
        &self,
        ctx: &impl GetClsSignatureCtx,
        param_types: &[ClassId],
        node: &Rc<ShipArgs<'src>>,
    ) -> Result<(ConsId, &ParamsSignature), ConsError<'src>> {
        self.iter()
            .filter_map(|(cons_id, cons_signature)| {
                let (is_match, degree) = cons_signature.matches(ctx, param_types);
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
    fn render(&self, _src: &impl crate::ByteSource<'src>) -> String {
        match self {
            ConsError::NoMatchingCons { args: _ } => {
                "No constructor overload matching argument types was found".to_string()
            },
        }
    }
}
impl<'src> WithParserLoc for ConsError<'src> {
    fn loc(&self) -> ParserLoc {
        match self {
            ConsError::NoMatchingCons { args } => args.loc(),
        }
    }
}

pub struct MethodSignature {
    pub params: ParamsSignature,
    pub return_type: Option<ClassId>,
    pub overriding: Option<(ClassId, MethodId)>, //top ClassId
}

#[derive(Debug, Clone, Display)]
pub enum MethodError<'src> {
    #[display("no method name found")]
    UndefinedMethod { name: Rc<ShipId<'src>> },
    #[display("no overload matching call")]
    NoOverload { args: Rc<ShipArgs<'src>> },
}
impl<'src> Renderable<'src> for MethodError<'src> {
    fn render(&self, _src: &impl crate::ByteSource<'src>) -> String {
        match self {
            MethodError::UndefinedMethod { name } => {
                format!("Method with name `{}` was not found", name.id)
            },
            MethodError::NoOverload { args: _ } => {
                "No overload matching argument types found".to_string()
            },
        }
    }
}
impl<'src> WithParserLoc for MethodError<'src> {
    fn loc(&self) -> ParserLoc {
        match self {
            MethodError::UndefinedMethod { name } => name.loc(),
            MethodError::NoOverload { args } => args.loc(),
        }
    }
}

pub type MethodSignatureRegistry = MethodRegistry<MethodSignature>;
pub type MethodSignatureRegistryBuilder = MethodRegistryBuilder<MethodSignature>;

pub type ConsSignatureRegistry = ConsRegistry<ParamsSignature>;
pub type ConsSignatureRegistryBuilder = ConsRegistryBuilder<ParamsSignature>;

pub struct ClassSignature {
    pub id: ClassId,
    pub parent: ClassId,
    pub constructors: ConsSignatureRegistry,
    pub methods: MethodSignatureRegistry,
}
pub const INVALID_CLS_SIGNATURE: ClassSignature = ClassSignature {
    id: ClassId::Invalid,
    parent: ClassId::Invalid,
    constructors: Registry::empty(),
    methods: Registry::empty(),
};

pub type ClassSignatureRegistry = Registry<UserClassId, ClassSignature>;

pub trait ClassSignatureCtx {
    fn signatures(&self) -> &ClassSignatureRegistry;
}
impl ClassSignatureCtx for ClassSignatureRegistry {
    fn signatures(&self) -> &ClassSignatureRegistry {
        self
    }
}

pub trait GetClsSignatureCtx: StdlibCtx + ClassSignatureCtx {
    fn get_cls_signature(&self, cls_id: &ClassId) -> &ClassSignature;
    fn is_cls_subcls_of<C: Into<ClassId>, P: Into<ClassId>>(
        &self,
        child: C,
        parent: P,
    ) -> (bool, u8);
    fn get_top_method(&self, cls: ClassId, method: MethodId) -> (ClassId, MethodId);
}
impl<Ctx: StdlibCtx + ClassSignatureCtx> GetClsSignatureCtx for Ctx {
    fn get_cls_signature(&self, cls_id: &ClassId) -> &ClassSignature {
        match cls_id {
            ClassId::User(user_class_id) => self.signatures().get(user_class_id),
            ClassId::Lib(lib_class_id) => self.stdlib().cls_signature(lib_class_id),
            ClassId::Invalid => self.stdlib().invalid_signature(),
        }
    }

    fn is_cls_subcls_of<C: Into<ClassId>, P: Into<ClassId>>(
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
            if current == LibClassId::Class.into() || current == ClassId::Invalid {
                return (false, diff);
            }
            let signature = self.get_cls_signature(&current);
            current = signature.parent;
            diff += 1;
        }
    }

    fn get_top_method(&self, cls: ClassId, method: MethodId) -> (ClassId, MethodId) {
        match self.get_cls_signature(&cls).methods.get_method(&method).overriding {
            Some((cls, method)) => self.get_top_method(cls, method),
            None => (cls, method),
        }
    }
}

pub trait FindMatchingMethodCtx<'src>:
    StdlibCtx + ClassSignatureCtx + ClassMemberNamesCtx<'src>
{
    fn find_matching_method(
        &self,
        cls_id: ClassId,
        name: &'src str,
        param_types: &[ClassId],
        name_node: &Rc<ShipId<'src>>,
        args_node: &Rc<ShipArgs<'src>>,
    ) -> Result<(ClassId, MethodId, &MethodSignature), MethodError<'src>>;
}
impl<'src, Ctx: StdlibCtx + ClassSignatureCtx + ClassMemberNamesCtx<'src>>
    FindMatchingMethodCtx<'src> for Ctx
{
    fn find_matching_method(
        &self,
        cls_id: ClassId,
        name: &'src str,
        param_types: &[ClassId],
        name_node: &Rc<ShipId<'src>>,
        args_node: &Rc<ShipArgs<'src>>,
    ) -> Result<(ClassId, MethodId, &MethodSignature), MethodError<'src>> {
        let signature = self.get_cls_signature(&cls_id);
        let methods = &signature.methods;

        let Some(name_id) = self.get_member_names(&cls_id).methods.get_by_name(name) else {
            let parent = signature.parent;
            if parent == LibClassId::Class.into() || parent == ClassId::Invalid {
                return Err(MethodError::UndefinedMethod { name: name_node.clone() });
            } else {
                return self.find_matching_method(parent, name, param_types, name_node, args_node);
            }
        };

        methods
            .get(&name_id)
            .iter()
            .filter_map(|(overload_id, method_signature)| {
                let (is_match, degree) = method_signature.params.matches(self, param_types);
                if is_match { Some((overload_id, degree)) } else { None }
            })
            .min_by(|fst, snd| fst.1.cmp(&snd.1))
            .map(|(overload_id, _degree)| {
                let method_id = MethodId::from((name_id, overload_id));
                Ok((cls_id, method_id, methods.get_method(&method_id)))
            })
            .unwrap_or_else(|| {
                let parent = signature.parent;
                if parent == LibClassId::Class.into() || parent == ClassId::Invalid {
                    Err(MethodError::NoOverload { args: args_node.clone() })
                } else {
                    match self.find_matching_method(parent, name, param_types, name_node, args_node)
                    {
                        Ok(res) => Ok(res),
                        Err(_) => Err(MethodError::NoOverload { args: args_node.clone() }),
                    }
                }
            })
    }
}

fn resolve_params_signature<'src>(
    names: &ClassNameRegistry<'src>,
    params_node: &Rc<ShipParams<'src>>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> ParamsSignature {
    ParamsSignature::new(
        params_node
            .params
            .iter()
            .map(|param| {
                let cls_name = &param.var_type;
                names.get_class_with_err(cls_name, errors)
            })
            .collect(),
    )
}

fn resolve_method_signature<'src>(
    names: &ClassNameRegistry<'src>,
    method_node: &Rc<ShipMethodDef<'src>>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> MethodSignature {
    let params = resolve_params_signature(names, &method_node.params, errors);
    let return_type =
        method_node.return_type.as_ref().map(|cls_name| names.get_class_with_err(cls_name, errors));
    MethodSignature { params, return_type, overriding: None }
}

pub fn init_cls_signature_registry<
    'src,
    Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>,
>(
    ctx: &Ctx,
    defs: &ClassDefsRegistry<'src>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> ClassSignatureRegistry {
    let signatures = defs
        .iter()
        .map(|(id, def)| {
            let mut parent = def
                .node
                .parent_id
                .as_ref()
                .map(|cls_name| ctx.cls_names().get_class_with_err(cls_name, errors))
                .unwrap_or(ClassId::Lib(LibClassId::AnyRef));

            if let ClassId::Lib(
                LibClassId::Class
                | LibClassId::AnyValue
                | LibClassId::Integer
                | LibClassId::Real
                | LibClassId::Boolean
                | LibClassId::Char,
            ) = parent
            {
                errors.push(
                    ClassError::InvalidInheritance { parent: def.node.parent_id.clone().unwrap() }
                        .into(),
                );
                parent = ClassId::Invalid;
            }

            let constructors = def
                .constructors
                .iter()
                .map(|(id, cons)| {
                    (id, resolve_params_signature(ctx.cls_names(), &cons.params, errors))
                })
                .collect();

            let methods = def.methods.map_method(|_method_id, method| {
                resolve_method_signature(ctx.cls_names(), method, errors)
            });
            (id, ClassSignature { id: id.into(), parent, constructors, methods })
        })
        .collect();
    let signatures = check_inheritance(signatures, defs, errors);
    check_main(&signatures, defs, ctx, errors);
    resolve_overrides(signatures, defs, ctx, errors)
}

enum VisitStatus {
    Fresh,
    Valid,
    Invalid,
}

fn check_parent<'src>(
    signatures: &ClassSignatureRegistry,
    defs: &ClassDefsRegistry<'src>,
    id: UserClassId,
    visited: &mut HashMap<UserClassId, VisitStatus>,
) -> Result<bool, CircularInheritance<'src>> {
    let data = signatures.get(&id);
    match visited.get(&id) {
        Some(VisitStatus::Valid) => Ok(true),
        Some(VisitStatus::Invalid) => Ok(false),
        Some(VisitStatus::Fresh) => {
            Err(CircularInheritance { chain: vec![defs.get(&id).node.clone()] })
        },
        None => {
            visited.insert(id, VisitStatus::Fresh);
            let res = match &data.parent {
                ClassId::User(parent_id) => check_parent(signatures, defs, *parent_id, visited)
                    .map_err(|mut e| {
                        e.chain.push(defs.get(&id).node.clone());
                        CircularInheritance { chain: e.chain }
                    }),
                ClassId::Lib(_lib_class_id) => Ok(true),
                ClassId::Invalid => Ok(true),
            };
            visited.insert(id, if res.is_ok() { VisitStatus::Valid } else { VisitStatus::Invalid });
            res
        },
    }
}

fn check_inheritance<'src>(
    signatures: ClassSignatureRegistry,
    defs: &ClassDefsRegistry<'src>,
    errors: &mut Vec<AnalysisError<'src>>,
) -> ClassSignatureRegistry {
    let mut visited: HashMap<UserClassId, VisitStatus> = HashMap::new();
    let statuses = signatures
        .iter()
        .map(|(id, _signature)| {
            (
                id,
                match check_parent(&signatures, defs, id, &mut visited) {
                    Ok(is_valid) => is_valid,
                    Err(e) => {
                        errors.push(ClassError::CircularInheritance(e).into());
                        false
                    },
                },
            )
        })
        .collect();
    signatures.combine(statuses, |signature, is_valid| {
        if is_valid { signature } else { ClassSignature { parent: ClassId::Invalid, ..signature } }
    })
}

struct WithSignatures<'a, 'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>> {
    signatures: &'a ClassSignatureRegistry,
    ctx: &'a Ctx,
    phantom: PhantomData<&'src str>,
}
impl<'a, 'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>> ClassSignatureCtx
    for WithSignatures<'a, 'src, Ctx>
{
    fn signatures(&self) -> &ClassSignatureRegistry {
        self.signatures
    }
}
impl<'a, 'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>> StdlibCtx
    for WithSignatures<'a, 'src, Ctx>
{
    fn stdlib(&self) -> &ShipStdLib {
        self.ctx.stdlib()
    }
}
impl<'a, 'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>> ClassNamesCtx<'src>
    for WithSignatures<'a, 'src, Ctx>
{
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        self.ctx.cls_names()
    }
}
impl<'a, 'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>>
    ClassMemberNamesCtx<'src> for WithSignatures<'a, 'src, Ctx>
{
    fn member_names(&self) -> &ClassMemberNamesRegistry<'src> {
        self.ctx.member_names()
    }
}

fn check_main<'src, Ctx: ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>>(
    signatures: &ClassSignatureRegistry,
    defs: &ClassDefsRegistry<'src>,
    ctx: &Ctx,
    errors: &mut Vec<AnalysisError<'src>>,
) {
    let Some((main_cls_id, main_cls)) =
        signatures.iter().find(|(id, _signature)| ctx.cls_names().get_name(id) == "Main")
    else {
        errors.push(ClassError::MissingMain.into());
        return;
    };
    if main_cls.constructors.len() != 1 {
        errors
            .push(ClassError::InvalidMainCons { main: defs.get(&main_cls_id).node.clone() }.into());
        return;
    }

    for (_cons_id, cons) in main_cls.constructors.iter() {
        if cons.param_types.len() != 1 || cons.param_types[0] != LibClassId::Array.into() {
            errors.push(
                ClassError::InvalidMainCons { main: defs.get(&main_cls_id).node.clone() }.into(),
            );
        }
    }
}

fn find_override<
    'src,
    Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src> + ClassSignatureCtx,
>(
    defs: &ClassDefsRegistry<'src>,
    ctx: &Ctx,
    cls_id: UserClassId,
    signature: &ClassSignature,
    method_id: MethodId,
    method_signature: &MethodSignature,
    errors: &mut Vec<AnalysisError<'src>>,
) -> Option<(ClassId, MethodId)> {
    let name = ctx.get_member_names(&cls_id.into()).methods.get_name(&method_id.0);
    let mut parent = signature.parent;
    while parent != ClassId::Invalid && parent != LibClassId::Class.into() {
        let parent_signature = ctx.get_cls_signature(&parent);
        let parent_methods = &parent_signature.methods;

        for (parent_name_id, overloads) in parent_methods {
            let parent_name = ctx.get_member_names(&parent).methods.get_name(&parent_name_id);
            if name == parent_name {
                for (parent_overload_id, overload) in overloads {
                    if method_signature.params == overload.params {
                        if method_signature.return_type == overload.return_type {
                            return Some((parent, (parent_name_id, parent_overload_id).into()));
                        } else {
                            errors.push(
                                ClassError::InvalidOverload {
                                    method: defs
                                        .get(&cls_id)
                                        .methods
                                        .get_method(&method_id)
                                        .clone(),
                                }
                                .into(),
                            );
                            return None;
                        }
                    }
                }
            }
        }
        parent = parent_signature.parent;
    }

    None
}

fn resolve_overrides<'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src> + ClassNamesCtx<'src>>(
    signatures: ClassSignatureRegistry,
    defs: &ClassDefsRegistry<'src>,
    ctx: &Ctx,
    errors: &mut Vec<AnalysisError<'src>>,
) -> ClassSignatureRegistry {
    let overrides = signatures
        .iter()
        .map(|(id, signature)| {
            let method_overrides = signature.methods.map_method(|method_id, method| {
                find_override(
                    defs,
                    &WithSignatures { signatures: &signatures, ctx, phantom: PhantomData },
                    id,
                    signature,
                    method_id,
                    method,
                    errors,
                )
            });
            (id, method_overrides)
        })
        .collect();

    signatures.combine(overrides, |signature, overrides| ClassSignature {
        methods: signature.methods.combine(overrides, |overloads, overrides| {
            overloads.combine(overrides, |overload, overriding| MethodSignature {
                overriding,
                ..overload
            })
        }),
        ..signature
    })
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
    #[display("invalid overload")]
    InvalidOverload { method: Rc<ShipMethodDef<'src>> },
    #[display("missing main")]
    MissingMain,
    #[display("invalid main cons")]
    InvalidMainCons { main: Rc<ShipClassDef<'src>> },
    #[display("invalid inheritance")]
    InvalidInheritance { parent: Rc<ShipId<'src>> },
}

impl<'src> Renderable<'src> for ClassError<'src> {
    fn render(&self, src: &impl crate::ByteSource<'src>) -> String {
        match self {
            ClassError::CircularInheritance(circular_inheritance) => format!(
                "Circular inheritance chain found:\n{}",
                circular_inheritance
                    .chain
                    .iter()
                    .map(|cls| src.source_str(
                        cls.parent_id
                            .as_ref()
                            .map(|parent| ParserLoc::merge(cls.class_id.raw_loc, parent.raw_loc))
                            .unwrap_or(cls.class_id.raw_loc)
                    ))
                    .join("\n")
            ),
            ClassError::DuplicateClassName { fst, snd } => format!(
                "Class with name `{}` is defined multiple times:\nFirst declaration is:\n{}\n{}\nSecond declaration is:\n{}\n{}",
                fst.class_id.id,
                fst.start,
                fst.src(),
                snd.start,
                snd.src()
            ),
            ClassError::DuplicateConstructor { fst, snd } => format!(
                "Constructor with same param types is defined multiple times:\nFirst declaration is:\n{}\n{}\nSecond declaration is:\n{}\n{}",
                fst.start,
                fst.src(),
                snd.start,
                snd.src()
            ),
            ClassError::DuplicateField { fst, snd } => format!(
                "Field with name `{}` is defined multiple times:\nFirst declaration is:\n{}\n{}\nSecond declaration is:\n{}\n{}",
                fst.var_id.id,
                fst.start,
                fst.src(),
                snd.start,
                snd.src()
            ),
            ClassError::InvalidOverload { method: _method } => {
                "Method's return type does not match return type of the method its overriding"
                    .to_string()
            },
            ClassError::MissingMain => "Program is missing a 'Main' class".to_string(),
            ClassError::InvalidMainCons { main: _main } => {
                "'Main' class should have a single constructor with an 'Array' argument".to_string()
            },
            ClassError::InvalidInheritance { parent } => {
                format!("Cannot inherit '{}'", parent.src())
            },
        }
    }
}

impl<'src> WithParserLoc for ClassError<'src> {
    fn loc(&self) -> ParserLoc {
        match self {
            ClassError::CircularInheritance(circular_inheritance) => {
                circular_inheritance.chain.first().unwrap().loc()
            },
            ClassError::DuplicateClassName { fst: _, snd } => snd.loc(),
            ClassError::DuplicateConstructor { fst: _, snd } => snd.loc(),
            ClassError::DuplicateField { fst: _, snd } => snd.loc(),
            ClassError::InvalidOverload { method } => method.loc(),
            ClassError::MissingMain => ParserLoc { begin: 0, end: 0 },
            ClassError::InvalidMainCons { main } => main.class_id.loc(),
            ClassError::InvalidInheritance { parent } => parent.loc(),
        }
    }
}
