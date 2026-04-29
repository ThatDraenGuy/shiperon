mod body;
mod method;
use std::collections::HashMap;

use inkwell::{
    AddressSpace,
    builder::Builder as LLVMBuilder,
    context::Context as LLVMContext,
    module::{Linkage, Module as LLVMModule},
    targets::{self, Target},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType},
    values::{FunctionValue, GlobalValue, PointerValue},
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
            ClassId, ClassNameRegistry, ClassRegistry, ConsId, ConsRegistry, FieldId,
            FieldRegistry, LibClassId, MethodId, MethodRegistry, UserClassId,
        },
        signature::{MethodSignature, ParamsSignature},
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
    stdlib_impl: StdLibImpl<'ctx>,
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
    fn get_top_method(&self, cls: &ClassId, method: &MethodId) -> (ClassId, MethodId);
}
impl<Ctx: StdlibCtx + ClassModelCtx> GetFieldModels for Ctx {
    fn field_models(&self, cls_id: &ClassId) -> &FieldModelRegistry {
        match cls_id {
            ClassId::User(user_class_id) => &self.cls_models().get(user_class_id).fields,
            ClassId::Lib(lib_class_id) => &self.stdlib().cls_fields(lib_class_id).registry,
            ClassId::Invalid => unreachable!(),
        }
    }
    fn get_top_method(&self, cls: &ClassId, method: &MethodId) -> (ClassId, MethodId) {
        let overriding = &match cls {
            ClassId::User(user_class_id) => {
                self.cls_models().get(user_class_id).methods.get_method(method).signature.overriding
            },
            ClassId::Lib(lib_class_id) => {
                self.stdlib().cls_signature(lib_class_id).methods.get_method(method).overriding
            },
            ClassId::Invalid => unreachable!(),
        };
        match overriding {
            Some((cls, method)) => self.get_top_method(cls, method),
            None => (*cls, *method),
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
                LibClassId::Class => self.ctx().i64_type().into(), //TODO think
                LibClassId::AnyValue => self.ctx().i64_type().into(), //TODO think
                _ => self.ctx().ptr_type(AddressSpace::default()).into(),
            },
            ClassId::Invalid => unreachable!(),
        }
    }
}

pub struct FieldImpl {
    struct_offset: u32,
}

pub struct MethodImpl<'ctx> {
    func: FunctionValue<'ctx>,
    vtable_offset: u64,
}

pub struct ConsImpl<'ctx> {
    func: FunctionValue<'ctx>,
}

pub struct ClassImpl<'ctx> {
    object_type: StructType<'ctx>,
    vtable_ptrs: Vec<PointerValue<'ctx>>,
    vtable: GlobalValue<'ctx>,
    init_func: FunctionValue<'ctx>,
    constructors: ConsRegistry<ConsImpl<'ctx>>,
    methods: MethodRegistry<MethodImpl<'ctx>>,
    fields: FieldRegistry<FieldImpl>,
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
    fn create_cons_name<'src>(
        ast: &impl ShipCtx<'src>,
        cls_id: &UserClassId,
        cons_id: ConsId,
    ) -> String {
        let cls_name = ast.cls_names().get_name(cls_id);
        let signature = &ast.cls_models().get(cls_id).constructors.get(&cons_id).signature;
        let mangle =
            signature.param_types.iter().map(|param_type| ast.cls_name(param_type)).join("_");
        format!("cls_{cls_name}_cons_args_{mangle}")
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
    fn codegen_cons_decl(
        llvm: &impl LLVMCtx<'ctx>,
        signature: &ParamsSignature,
        cons_name: &str,
    ) -> FunctionValue<'ctx> {
        let mut params: Vec<_> = signature
            .param_types
            .iter()
            .map(|param_type| BasicMetadataTypeEnum::from(llvm.get_value_type(param_type)))
            .collect();
        params.insert(
            0,
            BasicMetadataTypeEnum::PointerType(llvm.ctx().ptr_type(AddressSpace::default())), //ptr to self
        );
        let func_type = llvm.ctx().ptr_type(AddressSpace::default()).fn_type(&params, false);
        llvm.module().add_function(cons_name, func_type, None)
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

    fn get_cls_impl<'a>(
        stdlib: &'a StdLibImpl<'ctx>,
        ready: &'a HashMap<UserClassId, ClassImpl<'ctx>>,
        cls_id: &ClassId,
    ) -> &'a ClassImpl<'ctx> {
        match cls_id {
            ClassId::User(user_class_id) => ready.get(user_class_id).unwrap(),
            ClassId::Lib(lib_class_id) => stdlib.get(lib_class_id),
            ClassId::Invalid => unreachable!(),
        }
    }
    fn codegen_cls_globals<'src, 'a>(
        llvm: &impl LLVMCtx<'ctx>,
        ast: &impl ShipCtx<'src>,
        stdlib: &'a StdLibImpl<'ctx>,
        ready: &'a mut HashMap<UserClassId, ClassImpl<'ctx>>,
        cls_id: UserClassId,
        cls: &ClassModel,
    ) -> &'a ClassImpl<'ctx> {
        let parent_impl = match &cls.parent {
            ClassId::User(user_parent) => match ready.get(user_parent) {
                Some(parent_impl) => parent_impl,
                None => Self::codegen_cls_globals(
                    llvm,
                    ast,
                    stdlib,
                    ready,
                    *user_parent,
                    ast.cls_models().get(user_parent),
                ),
            },
            ClassId::Lib(lib_class_id) => stdlib.get(lib_class_id),
            ClassId::Invalid => unreachable!(),
        };
        let parent_obj_type = parent_impl.object_type;

        //class object struct type
        let mut struct_member_types = Vec::new();

        struct_member_types.push(parent_obj_type.into()); //parent obj type
        let fields = cls
            .fields
            .iter()
            .map(|(field_id, field)| {
                let field_type = llvm.get_value_type(&field.field_type);
                let offset = struct_member_types.len();
                struct_member_types.push(field_type);
                (field_id, FieldImpl { struct_offset: offset as u32 })
            })
            .collect();
        let object_type = Self::get_user_struct_type(llvm, ast, &cls_id);
        object_type.set_body(&struct_member_types, true);

        let constructors = cls
            .constructors
            .iter()
            .map(|(cons_id, cons)| {
                let func = Self::codegen_cons_decl(
                    llvm,
                    &cons.signature,
                    &Self::create_cons_name(ast, &cls_id, cons_id),
                );
                (cons_id, ConsImpl { func })
            })
            .collect();

        //class vtable
        let mut vtable_ptrs = parent_impl.vtable_ptrs.clone();
        let methods = cls.methods.map_method(|method_id, method| {
            let func = Self::codegen_method_decl(
                llvm,
                &method.signature,
                &Self::create_method_name(ast, &cls_id, method_id),
            );
            let vtable_offset = match &method.signature.overriding {
                Some((parent_id, parent_method_id)) => {
                    let offset = Self::get_cls_impl(stdlib, ready, parent_id)
                        .methods
                        .get_method(parent_method_id)
                        .vtable_offset;
                    vtable_ptrs[offset as usize] = func.as_global_value().as_pointer_value();
                    offset
                },
                None => {
                    let offset = vtable_ptrs.len();
                    vtable_ptrs.push(func.as_global_value().as_pointer_value());
                    offset as u64
                },
            };
            MethodImpl { func, vtable_offset }
        });

        let vtable_type =
            llvm.ctx().ptr_type(AddressSpace::default()).array_type(vtable_ptrs.len() as u32);
        let vtable = llvm.module().add_global(
            vtable_type.as_basic_type_enum(),
            None,
            &format!("cls_{}_vtable_data", ast.cls_names().get_name(&cls_id)),
        );
        vtable.set_constant(true);
        vtable.set_initializer(
            &llvm.ctx().ptr_type(AddressSpace::default()).const_array(&vtable_ptrs),
        );

        let init_func_type = llvm
            .ctx()
            .void_type()
            .fn_type(&[llvm.ctx().ptr_type(AddressSpace::default()).into()], false);
        let init_func = llvm.module().add_function(
            &format!("cls_{}_init", ast.cls_names().get_name(&cls_id)),
            init_func_type,
            None,
        );

        ready.insert(
            cls_id,
            ClassImpl {
                vtable,
                vtable_ptrs,
                init_func,
                object_type,
                constructors,
                methods,
                fields,
            },
        );
        &ready[&cls_id]
    }

    pub fn new<'src>(
        llvm: &impl LLVMCtx<'ctx>,
        ast: &impl ShipCtx<'src>,
        stdlib: &StdLibImpl<'ctx>,
    ) -> Self {
        let mut ready = HashMap::new();
        for (cls_id, cls) in ast.cls_models() {
            Self::codegen_cls_globals(llvm, ast, stdlib, &mut ready, cls_id, cls);
        }
        ast.cls_models()
            .iter()
            .map(|(cls_id, _cls)| (cls_id, ready.remove(&cls_id).unwrap()))
            .collect()
    }
}

pub struct StdLibImpl<'ctx> {
    impls: HashMap<LibClassId, ClassImpl<'ctx>>,
    malloc: FunctionValue<'ctx>,
}
impl<'ctx> StdLibImpl<'ctx> {
    fn codegen(
        llvm: &ShipLLVMContext<'ctx>,
        stdlib: &ShipStdLib,
        ready: &mut HashMap<LibClassId, ClassImpl<'ctx>>,
        cls_id: &LibClassId,
    ) -> ClassImpl<'ctx> {
        let ClassId::Lib(parent_id) = stdlib.cls_signature(cls_id).parent else { unreachable!() };
        let lib_impl = stdlib.cls_impl(cls_id);

        let mut struct_member_types = Vec::new();
        if *cls_id == LibClassId::Class {
            struct_member_types.push(llvm.ctx.ptr_type(AddressSpace::default()).into()); //vtable ptr
        } else {
            let parent_obj_type = ready[&parent_id].object_type;
            struct_member_types.push(parent_obj_type.into()); //parent obj type
        };

        let fields = stdlib
            .cls_fields(cls_id)
            .registry
            .iter()
            .map(|(field_id, field)| {
                let field_type = llvm.get_value_type(&field.field_type);
                let offset = struct_member_types.len();
                struct_member_types.push(field_type);
                (field_id, FieldImpl { struct_offset: offset as u32 })
            })
            .collect();
        let object_type = llvm.ctx.opaque_struct_type(stdlib.cls_name(cls_id));
        object_type.set_body(&struct_member_types, true);

        let constructors = lib_impl
            .constructors
            .iter()
            .map(|(cons_id, cons)| {
                let func = (cons.def_impl)(llvm).unwrap();
                (cons_id, ConsImpl { func })
            })
            .collect();

        let mut vtable_ptrs = if *cls_id == LibClassId::Class {
            Vec::new()
        } else {
            ready[&parent_id].vtable_ptrs.clone()
        };
        let methods = lib_impl.methods.map_method(|_method_id, method| {
            let func = (method.def_impl)(llvm).unwrap();
            let vtable_offset = vtable_ptrs.len() as u64;
            vtable_ptrs.push(func.as_global_value().as_pointer_value());
            MethodImpl { func, vtable_offset }
        });

        let vtable_type =
            llvm.ctx().ptr_type(AddressSpace::default()).array_type(vtable_ptrs.len() as u32);
        let vtable = llvm.module().add_global(
            vtable_type.as_basic_type_enum(),
            None,
            &format!("cls_{}_vtable_data", stdlib.cls_name(cls_id)),
        );
        vtable.set_constant(true);
        vtable.set_initializer(
            &llvm.ctx().ptr_type(AddressSpace::default()).const_array(&vtable_ptrs),
        );

        let init_func = (lib_impl.init_impl)(llvm).unwrap();

        ClassImpl { object_type, vtable_ptrs, vtable, init_func, constructors, methods, fields }
    }

    pub fn new(llvm: &ShipLLVMContext<'ctx>, stdlib: &ShipStdLib) -> Self {
        let malloc_type = llvm
            .ctx
            .ptr_type(AddressSpace::default())
            .fn_type(&[llvm.ctx.i64_type().into()], false);
        let malloc = llvm.module.add_function("GC_malloc", malloc_type, Some(Linkage::External));

        let mut impls = HashMap::new();
        let class_impl = Self::codegen(llvm, stdlib, &mut impls, &LibClassId::Class);
        impls.insert(LibClassId::Class, class_impl);
        let anyref_impl = Self::codegen(llvm, stdlib, &mut impls, &LibClassId::AnyRef);
        impls.insert(LibClassId::AnyRef, anyref_impl);

        Self { impls, malloc }
    }
    pub fn get(&self, cls_id: &LibClassId) -> &ClassImpl<'ctx> {
        self.impls.get(cls_id).unwrap()
    }
    pub fn malloc(&self) -> FunctionValue<'ctx> {
        self.malloc
    }
}

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    pub fn new(ast: ShipContext<'src>, ctx: &'ctx LLVMContext) -> Self {
        let builder = ctx.create_builder();
        let module = ctx.create_module("ship_module");

        let llvm = ShipLLVMContext { ctx, builder, module };

        let stdlib_impl = StdLibImpl::new(&llvm, ast.stdlib());
        let impls = ClassImplRegistry::new(&llvm, &ast, &stdlib_impl);
        Self { ast, llvm, impls, stdlib_impl }
    }

    pub fn codegen(&self) {
        for (cls_id, cls) in self.ast.cls_models() {
            for (name_id, overloads) in &cls.methods {
                for (overload_id, method) in overloads {
                    self.codegen_method(&cls_id, (name_id, overload_id).into(), method);
                }
            }
        }
    }
}

pub fn compile<'src>(ast: ShipContext<'src>) {
    let target_config = targets::InitializationConfig::default();
    Target::initialize_native(&target_config).expect("Failed to initialize native machine target!");

    Target::initialize_all(&target_config);
    let ctx = inkwell::context::Context::create();

    let codegen = CodegenContext::new(ast, &ctx);
    codegen.codegen();
    println!("{}", codegen.llvm.module.print_to_string().to_string());
}
