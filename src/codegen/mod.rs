mod body;
mod method;
use inkwell::{
    AddressSpace, OptimizationLevel,
    builder::Builder as LLVMBuilder,
    context::Context as LLVMContext,
    module::Module as LLVMModule,
    targets::{self, CodeModel, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType},
    values::FunctionValue,
};
use itertools::Itertools;

use crate::{
    ShipStdLib, StdlibCtx,
    analyzer::{
        ShipContext, ShipCtx,
        def::{ClassMemberNamesCtx, ClassMemberNamesRegistry, ClassNamesCtx},
        field::FieldModelRegistry,
        model::{ClassModel, ClassModelCtx, ClassModelRegistry},
        registry::{
            ClassId, ClassNameRegistry, ClassRegistry, FieldId, FieldRegistry, LibClassId,
            MethodId, MethodRegistry, UserClassId,
        },
        signature::MethodSignature,
    },
};

pub struct ShipLLVMContext<'ctx> {
    ctx: &'ctx LLVMContext,
    builder: LLVMBuilder<'ctx>,
    module: LLVMModule<'ctx>,
}

pub trait LLVMCtx<'ctx> {
    fn ctx(&self) -> &'ctx LLVMContext;
    fn builder(&self) -> &LLVMBuilder<'ctx>;
    fn module(&self) -> &LLVMModule<'ctx>;
}
impl<'ctx> LLVMCtx<'ctx> for ShipLLVMContext<'ctx> {
    fn ctx(&self) -> &'ctx LLVMContext {
        self.ctx
    }

    fn builder(&self) -> &LLVMBuilder<'ctx> {
        &self.builder
    }

    fn module(&self) -> &LLVMModule<'ctx> {
        &self.module
    }
}

pub struct CodegenContext<'ctx, 'src> {
    ast: ShipContext<'src>,
    llvm: ShipLLVMContext<'ctx>,
    impls: ClassRegistry<ClassImpl<'ctx>>,
}
impl<'ctx, 'src> StdlibCtx for CodegenContext<'ctx, 'src> {
    fn stdlib(&self) -> &ShipStdLib {
        self.ast.stdlib()
    }
}
impl<'ctx, 'src> ClassNamesCtx<'src> for CodegenContext<'ctx, 'src> {
    fn cls_names(&self) -> &ClassNameRegistry<'src> {
        self.ast.cls_names()
    }
}
impl<'ctx, 'src> ClassMemberNamesCtx<'src> for CodegenContext<'ctx, 'src> {
    fn member_names(&self) -> &ClassMemberNamesRegistry<'src> {
        self.ast.member_names()
    }
}
impl<'ctx, 'src> ClassModelCtx for CodegenContext<'ctx, 'src> {
    fn cls_models(&self) -> &ClassModelRegistry {
        self.ast.cls_models()
    }
}
impl<'ctx, 'src> ClassImplCtx<'ctx> for CodegenContext<'ctx, 'src> {
    fn impls(&self) -> &ClassImplRegistry<'ctx> {
        &self.impls
    }
}
impl<'ctx, 'src> LLVMCtx<'ctx> for CodegenContext<'ctx, 'src> {
    fn ctx(&self) -> &'ctx LLVMContext {
        self.llvm.ctx()
    }

    fn builder(&self) -> &LLVMBuilder<'ctx> {
        self.llvm.builder()
    }

    fn module(&self) -> &LLVMModule<'ctx> {
        self.llvm.module()
    }
}

pub trait GetClsNameCtx<'src>: StdlibCtx + ClassNamesCtx<'src> {
    fn cls_name(&self, cls_id: &ClassId) -> &'src str;
}
impl<'src, Ctx: StdlibCtx + ClassNamesCtx<'src>> GetClsNameCtx<'src> for Ctx {
    fn cls_name(&self, cls_id: &ClassId) -> &'src str {
        match cls_id {
            ClassId::User(user_class_id) => self.cls_names().get_name(user_class_id),
            ClassId::Lib(lib_class_id) => self.stdlib().cls_name(lib_class_id),
            ClassId::Invalid => unreachable!(),
        }
    }
}

pub trait GetFieldNameCtx<'src>: StdlibCtx + ClassMemberNamesCtx<'src> {
    fn field_name(&self, cls_id: &ClassId, field_id: &FieldId) -> &'src str;
}
impl<'src, Ctx: StdlibCtx + ClassMemberNamesCtx<'src>> GetFieldNameCtx<'src> for Ctx {
    fn field_name(&self, cls_id: &ClassId, field_id: &FieldId) -> &'src str {
        match cls_id {
            ClassId::User(user_class_id) => {
                self.member_names().get(user_class_id).fields.get_name(field_id)
            },
            ClassId::Lib(lib_class_id) => {
                self.stdlib().cls_member_names(lib_class_id).fields.get_name(field_id)
            },
            ClassId::Invalid => unreachable!(),
        }
    }
}

pub trait GetFieldModels: StdlibCtx + ClassModelCtx {
    fn field_models(&self, cls_id: &ClassId) -> &FieldModelRegistry;
}
impl<Ctx: StdlibCtx + ClassModelCtx> GetFieldModels for Ctx {
    fn field_models(&self, cls_id: &ClassId) -> &FieldModelRegistry {
        match cls_id {
            ClassId::User(user_class_id) => &self.cls_models().get(user_class_id).fields,
            ClassId::Lib(lib_class_id) => &self.stdlib().cls_fields(lib_class_id).registry,
            ClassId::Invalid => unreachable!(),
        }
    }
}

pub trait GetValueType<'ctx>: LLVMCtx<'ctx> {
    fn get_value_type(&self, cls_id: &ClassId) -> BasicTypeEnum<'ctx>;
    // fn get_struct_type(&self, cls_id: &ClassId) -> StructType<'ctx>;
}
impl<'ctx, Ctx: LLVMCtx<'ctx>> GetValueType<'ctx> for Ctx {
    fn get_value_type(&self, cls_id: &ClassId) -> BasicTypeEnum<'ctx> {
        match cls_id {
            ClassId::User(_) => {
                BasicTypeEnum::PointerType(self.ctx().ptr_type(AddressSpace::default()))
            },
            ClassId::Lib(lib_class_id) => match lib_class_id {
                LibClassId::Integer => self.ctx().i32_type().into(),
                LibClassId::Real => self.ctx().f32_type().into(),
                LibClassId::Boolean => self.ctx().bool_type().into(),
                LibClassId::Char => self.ctx().i8_type().into(),
                LibClassId::Class => todo!(),
                LibClassId::AnyValue => todo!(),
                _ => self.ctx().ptr_type(AddressSpace::default()).into(),
            },
            ClassId::Invalid => unreachable!(),
        }
    }
}

pub struct FielImpl {
    struct_offset: u32,
}

pub struct MethodImpl<'ctx> {
    func: FunctionValue<'ctx>,
    vtable_offset: Option<u32>,
}

pub struct ClassImpl<'ctx> {
    vtable_type: StructType<'ctx>,
    object_type: StructType<'ctx>,
    methods: MethodRegistry<MethodImpl<'ctx>>,
    fields: FieldRegistry<FielImpl>,
}
pub type ClassImplRegistry<'ctx> = ClassRegistry<ClassImpl<'ctx>>;

pub trait ClassImplCtx<'ctx> {
    fn impls(&self) -> &ClassImplRegistry<'ctx>;
}
impl<'ctx> ClassImplCtx<'ctx> for ClassImplRegistry<'ctx> {
    fn impls(&self) -> &ClassImplRegistry<'ctx> {
        self
    }
}

impl<'ctx> ClassImplRegistry<'ctx> {
    fn create_method_name<'src>(
        ast: &impl ShipCtx<'src>,
        cls_id: &UserClassId,
        method_id: MethodId,
    ) -> String {
        let cls_name = ast.cls_names().get_name(cls_id);
        let method_name = ast.member_names().get(cls_id).methods.get_name(&method_id.0);
        let signature = &ast.cls_models().get(cls_id).methods.get_method(&method_id).signature;
        let mangle = signature
            .params
            .param_types
            .iter()
            .map(|param_type| ast.cls_name(param_type))
            .join("_");

        format!("cls_{cls_name}_method_{method_name}_args_{mangle}")
    }

    fn codegen_method_decl(
        llvm: &impl LLVMCtx<'ctx>,
        signature: &MethodSignature,
        method_name: &str,
    ) -> FunctionValue<'ctx> {
        let mut params: Vec<_> = signature
            .params
            .param_types
            .iter()
            .map(|param_type| BasicMetadataTypeEnum::from(llvm.get_value_type(param_type)))
            .collect();
        params.insert(
            0,
            BasicMetadataTypeEnum::PointerType(llvm.ctx().ptr_type(AddressSpace::default())), //ptr to self
        );
        let func_type = match signature.return_type {
            Some(return_type) => llvm.get_value_type(&return_type).fn_type(&params, false),
            None => llvm.ctx().void_type().fn_type(&params, false),
        };
        llvm.module().add_function(method_name, func_type, None)
    }

    fn get_struct_type<'src>(
        llvm: &impl LLVMCtx<'ctx>,
        ast: &impl ShipCtx<'src>,
        cls_id: &ClassId,
    ) -> StructType<'ctx> {
        match cls_id {
            ClassId::User(user_class_id) => Self::get_user_struct_type(llvm, ast, user_class_id),
            ClassId::Lib(lib_class_id) => todo!(),
            ClassId::Invalid => todo!(),
        }
    }

    fn get_user_struct_type<'src>(
        llvm: &impl LLVMCtx<'ctx>,
        ast: &impl ShipCtx<'src>,
        cls_id: &UserClassId,
    ) -> StructType<'ctx> {
        let struct_name = ast.cls_names().get_name(cls_id);
        match llvm.ctx().get_struct_type(struct_name) {
            Some(struct_type) => struct_type,
            None => llvm.ctx().opaque_struct_type(struct_name),
        }
    }

    fn codegen_cls<'src>(
        llvm: &impl LLVMCtx<'ctx>,
        ast: &impl ShipCtx<'src>,
        cls_id: UserClassId,
        cls: &ClassModel,
    ) -> ClassImpl<'ctx> {
        let object_type = Self::get_user_struct_type(llvm, ast, &cls_id);

        let mut struct_member_types = Vec::new();
        let parent_obj_type = Self::get_struct_type(llvm, ast, &cls.parent);
        struct_member_types.push(parent_obj_type.into()); //parent obj type
        struct_member_types.push(llvm.ctx().ptr_type(AddressSpace::default()).into()); //cls vtable ptr field
        let fields = cls
            .fields
            .iter()
            .map(|(field_id, field)| {
                let field_type = llvm.get_value_type(&field.field_type);
                let offset = struct_member_types.len();
                struct_member_types.push(field_type);
                (field_id, FielImpl { struct_offset: offset as u32 })
            })
            .collect();
        object_type.set_body(&struct_member_types, true);

        let mut vtable_member_types = Vec::new();

        let methods = cls
            .methods
            .iter()
            .map(|(name_id, overloads)| {
                (
                    name_id,
                    overloads
                        .iter()
                        .map(|(overload_id, method)| {
                            let func = Self::codegen_method_decl(
                                llvm,
                                &method.signature,
                                &Self::create_method_name(
                                    ast,
                                    &cls_id,
                                    (name_id, overload_id).into(),
                                ),
                            );
                            let vtable_offset = match &method.signature.overriding {
                                Some(_) => None,
                                None => {
                                    let offset = vtable_member_types.len();
                                    vtable_member_types
                                        .push(llvm.ctx().ptr_type(AddressSpace::default()).into());
                                    Some(offset as u32)
                                },
                            };
                            (overload_id, MethodImpl { func, vtable_offset })
                        })
                        .collect(),
                )
            })
            .collect();
        let vtable_type = llvm.ctx().struct_type(&vtable_member_types, true);
        ClassImpl { vtable_type, object_type, methods, fields }
    }

    pub fn new<'src>(llvm: &impl LLVMCtx<'ctx>, ast: &impl ShipCtx<'src>) -> Self {
        ast.cls_models()
            .iter()
            .map(|(cls_id, cls)| (cls_id, Self::codegen_cls(llvm, ast, cls_id, cls)))
            .collect()
    }
}

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    pub fn new(ast: ShipContext<'src>, ctx: &'ctx LLVMContext) -> Self {
        let builder = ctx.create_builder();
        let module = ctx.create_module("ship_module");

        let llvm_ctx = ShipLLVMContext { ctx, builder, module };

        let impls = ClassImplRegistry::new(&llvm_ctx, &ast);
        Self { ast, llvm: llvm_ctx, impls }
    }

    fn create_vtable(&self, cls_id: UserClassId, model: &ClassModel) -> () {
        let vtable_type = self.impls().get(&cls_id).vtable_type;

        let method_types: Vec<_> = model
            .methods
            .iter()
            .flat_map(|(name_id, overloads)| {
                overloads.iter().map(move |(overload_id, method)| (name_id, overload_id, method))
            })
            .map(|(name_id, overload_id, method)| {
                let mut params: Vec<_> = method
                    .signature
                    .params
                    .param_types
                    .iter()
                    .map(|param_type| BasicMetadataTypeEnum::from(self.get_value_type(param_type)))
                    .collect();
                params.insert(
                    0,
                    BasicMetadataTypeEnum::PointerType(
                        self.ctx().ptr_type(AddressSpace::default()),
                    ), //ptr to self
                );
                match method.signature.return_type {
                    Some(return_type) => self.get_value_type(&return_type).fn_type(&params, false),
                    None => self.ctx().void_type().fn_type(&params, false),
                }
            })
            .collect();

        // self.ctx.struct_type(field_types, packed)
        // let global = self.module.add_global(type_, address_space, name);
        todo!()
    }

    pub fn codegen(&self) {
        let res: Vec<_> = self
            .ast
            .cls_models()
            .iter()
            .map(|(cls_id, model)| self.create_vtable(cls_id, model))
            .collect();
        todo!()
    }
}

pub fn compile<'src>(ast: ShipContext<'src>) {
    let target_config = targets::InitializationConfig::default();
    Target::initialize_native(&target_config).expect("Failed to initialize native machine target!");

    Target::initialize_all(&target_config);
    let ctx = inkwell::context::Context::create();

    let codegen = CodegenContext::new(ast, &ctx);
    codegen.codegen();
}
