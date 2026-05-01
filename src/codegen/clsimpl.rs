use derive_more::{From, TryUnwrap, Unwrap};
use inkwell::{
    AddressSpace,
    types::StructType,
    values::{
        AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, GlobalValue,
        PointerValue,
    },
};

use crate::{
    analyzer::registry::{
        ClassRegistry, ConsId, ConsRegistry, FieldId, FieldRegistry, MethodId, MethodRegistry,
    },
    codegen::LLVMCtx,
};

pub struct FieldImpl {
    pub struct_offset: u32,
}

pub struct MethodImpl<'ctx> {
    pub func: FunctionValue<'ctx>,
    pub vtable_offset: u64,
}

pub struct ConsImpl<'ctx> {
    pub func: FunctionValue<'ctx>,
}

pub struct ObjectImpl<'ctx> {
    pub object_type: StructType<'ctx>,
    pub vtable_ptrs: Vec<PointerValue<'ctx>>,
    pub vtable: GlobalValue<'ctx>,
    pub init_func: FunctionValue<'ctx>,
    pub constructors: ConsRegistry<ConsImpl<'ctx>>,
    pub methods: MethodRegistry<MethodImpl<'ctx>>,
    pub fields: FieldRegistry<FieldImpl>,
}
impl<'ctx> ObjectImpl<'ctx> {
    pub fn get_field(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        object: BasicValueEnum<'ctx>,
        field_id: FieldId,
    ) -> PointerValue<'ctx> {
        let obj_ptr = object.into_pointer_value();
        let field_impl = self.fields.get(&field_id);

        llvm.builder()
            .build_struct_gep(
                self.object_type,
                obj_ptr,
                field_impl.struct_offset,
                &format!("load_{field_id}"),
            )
            .expect("FATAL: LLVM failed to build_struct_gep")
    }
    pub fn call_cons(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        cons_id: ConsId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        let cons_impl = self.constructors.get(&cons_id);
        let cons_func = cons_impl.func;

        let meta_args: Vec<_> = args.into_iter().map(BasicMetadataValueEnum::from).collect();
        let call_res = llvm
            .builder()
            .build_call(cons_func, &meta_args, "cons")
            .expect("FATAL: LLVM failed to build_call");
        call_res.try_as_basic_value().unwrap_basic()
    }
    pub fn call_method(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        object: BasicValueEnum<'ctx>,
        method_id: MethodId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> AnyValueEnum<'ctx> {
        let method_impl = self.methods.get_method(&method_id);

        let vtable_ptr = llvm
            .builder()
            .build_load(
                llvm.ctx().ptr_type(AddressSpace::default()),
                object.into_pointer_value(),
                "vtable_ptr",
            )
            .expect("FATAL: LLVM failed to build_load");
        //SAFETY: vtable should be safe (:
        let method_ptr_ptr = unsafe {
            llvm.builder()
                .build_gep(
                    llvm.ctx().ptr_type(AddressSpace::default()),
                    vtable_ptr.into_pointer_value(),
                    &[llvm.ctx().i32_type().const_int(method_impl.vtable_offset, false)],
                    "method_ptr_ptr",
                )
                .expect("FATAL: LLVM failed to build_gep")
        };
        let method_ptr = llvm
            .builder()
            .build_load(llvm.ctx().ptr_type(AddressSpace::default()), method_ptr_ptr, "method_ptr")
            .expect("FATAL: LLVM failed to build_load");

        let meta_args: Vec<_> = args.into_iter().map(BasicMetadataValueEnum::from).collect();
        let res = llvm
            .builder()
            .build_indirect_call(
                method_impl.func.get_type(),
                method_ptr.into_pointer_value(),
                &meta_args,
                "call",
            )
            .expect("FATAL: LLVM failed to build_inderect_call");
        res.as_any_value_enum()
    }
}

pub type ValueMethodImpl = for<'ctx> fn(
    ctx: &dyn LLVMCtx<'ctx>,
    object: BasicValueEnum<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> AnyValueEnum<'ctx>;
pub type ValueConsImpl =
    for<'ctx> fn(ctx: &dyn LLVMCtx<'ctx>, args: &[BasicValueEnum<'ctx>]) -> BasicValueEnum<'ctx>;

pub struct ValueImpl {
    pub methods: MethodRegistry<ValueMethodImpl>,
    pub constructors: ConsRegistry<ValueConsImpl>,
}
impl<'ctx> ValueImpl {
    pub fn call_cons(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        cons_id: ConsId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        self.constructors.get(&cons_id)(llvm, &args)
    }
    pub fn call_method(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        object: BasicValueEnum<'ctx>,
        method_id: MethodId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> AnyValueEnum<'ctx> {
        self.methods.get_method(&method_id)(llvm, object, &args)
    }
}

pub struct BlanketImpl;

#[derive(From, Unwrap)]
#[unwrap(ref)]
pub enum ClassImpl<'ctx> {
    Object(ObjectImpl<'ctx>),
    Value(ValueImpl),
    Blanket(BlanketImpl),
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

impl<'ctx> ClassImpl<'ctx> {
    pub fn init_func(&self) -> Option<FunctionValue<'ctx>> {
        match self {
            ClassImpl::Object(object_impl) => Some(object_impl.init_func),
            _ => None,
        }
    }
    pub fn get_field(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        object: BasicValueEnum<'ctx>,
        field_id: FieldId,
    ) -> PointerValue<'ctx> {
        match self {
            ClassImpl::Object(object_impl) => object_impl.get_field(llvm, object, field_id),
            ClassImpl::Value(value_impl) => todo!(),
            ClassImpl::Blanket(blanket_impl) => unreachable!(),
        }
    }
    pub fn call_cons(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        cons_id: ConsId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        match self {
            ClassImpl::Object(object_impl) => object_impl.call_cons(llvm, cons_id, args),
            ClassImpl::Value(value_impl) => value_impl.call_cons(llvm, cons_id, args),
            ClassImpl::Blanket(blanket_impl) => unreachable!(),
        }
    }

    pub fn call_method(
        &self,
        llvm: &impl LLVMCtx<'ctx>,
        object: BasicValueEnum<'ctx>,
        method_id: MethodId,
        args: Vec<BasicValueEnum<'ctx>>,
    ) -> AnyValueEnum<'ctx> {
        match self {
            ClassImpl::Object(object_impl) => {
                object_impl.call_method(llvm, object, method_id, args)
            },
            ClassImpl::Value(value_impl) => value_impl.call_method(llvm, object, method_id, args),
            ClassImpl::Blanket(blanket_impl) => unreachable!(),
        }
    }
}
