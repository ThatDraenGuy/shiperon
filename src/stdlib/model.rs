use std::collections::HashMap;

use derive_more::From;
use inkwell::{
    AddressSpace, FloatPredicate, IntPredicate,
    values::{AnyValueEnum, BasicValueEnum, FloatValue, FunctionValue, IntValue},
};

use crate::{
    analyzer::{
        field::FieldModel,
        registry::LibClassId,
        signature::{MethodSignature, ParamsSignature},
    },
    codegen::{
        LLVMCtx,
        clsimpl::{ValueConsImpl, ValueMethodImpl},
    },
};

pub struct LibConsObjectImpl {
    pub def_impl: for<'ctx> fn(&dyn LLVMCtx<'ctx>) -> FunctionValue<'ctx>,
}
pub struct LibMethodObjectImpl {
    pub def_impl: for<'ctx> fn(&dyn LLVMCtx<'ctx>) -> FunctionValue<'ctx>,
}
pub struct LibObjectModel {
    pub id: LibClassId,
    pub parent: LibClassId,
    pub init_impl: for<'ctx> fn(&dyn LLVMCtx<'ctx>) -> FunctionValue<'ctx>,
    pub constructors: Vec<(ParamsSignature, LibConsObjectImpl)>,
    pub methods: HashMap<&'static str, Vec<(MethodSignature, LibMethodObjectImpl)>>,
    pub fields: HashMap<&'static str, FieldModel>,
}

pub struct LibConsValueImpl {
    pub call_impl: ValueConsImpl,
}
pub struct LibMethodValueImpl {
    pub call_impl: ValueMethodImpl,
}
pub struct LibValueModel {
    pub id: LibClassId,
    pub parent: LibClassId,
    pub constructors: Vec<(ParamsSignature, LibConsValueImpl)>,
    pub methods: HashMap<&'static str, Vec<(MethodSignature, LibMethodValueImpl)>>,
    pub fields: HashMap<&'static str, FieldModel>,
}

pub struct LibBlanketModel {
    pub id: LibClassId,
    pub parent: LibClassId,
    pub constructors: Vec<ParamsSignature>,
    pub methods: HashMap<&'static str, Vec<MethodSignature>>,
    pub fields: HashMap<&'static str, FieldModel>,
}

#[derive(From)]
pub enum LibClassModel {
    Object(LibObjectModel),
    Value(LibValueModel),
    Blanket(LibBlanketModel),
}
impl LibClassModel {
    pub fn id(&self) -> LibClassId {
        match self {
            LibClassModel::Object(lib_object_model) => lib_object_model.id,
            LibClassModel::Value(lib_value_model) => lib_value_model.id,
            LibClassModel::Blanket(lib_blanket_model) => lib_blanket_model.id,
        }
    }
}

fn int_compare<'ctx>(
    predicate: IntPredicate,
    ctx: &dyn LLVMCtx<'ctx>,
    object: BasicValueEnum<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> AnyValueEnum<'ctx> {
    let left = object.into_int_value();
    let right = args.first().expect("FATAL: no args").into_int_value();
    let cmp_res = ctx
        .builder()
        .build_int_compare(predicate, left, right, "Less")
        .expect("FATAL: LLVM failed to build_int_compare");
    ctx.builder()
        .build_int_cast(cmp_res, ctx.ctx().bool_type(), "ToBoolean")
        .expect("FATAL: LLVM failed to build_int_cast")
        .into()
}

fn int_to_float<'ctx>(ctx: &dyn LLVMCtx<'ctx>, i: IntValue<'ctx>) -> FloatValue<'ctx> {
    ctx.builder()
        .build_signed_int_to_float(i, ctx.ctx().f32_type(), "ToReal")
        .expect("FATAL: LLVM failed to build_sitf")
}

fn float_to_int<'ctx>(ctx: &dyn LLVMCtx<'ctx>, f: FloatValue<'ctx>) -> IntValue<'ctx> {
    ctx.builder()
        .build_float_to_signed_int(f, ctx.ctx().i32_type(), "ToInteger")
        .expect("FATAL: LLVM failed to build_ftsi")
}

fn float_compare<'ctx>(
    predicate: FloatPredicate,
    ctx: &dyn LLVMCtx<'ctx>,
    object: BasicValueEnum<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> AnyValueEnum<'ctx> {
    let left = object.into_float_value();
    let right = args.first().expect("FATAL: no args").into_float_value();
    let cmp_res = ctx
        .builder()
        .build_float_compare(predicate, left, right, "Less")
        .expect("FATAL: LLVM failed to build_float_compare");
    ctx.builder()
        .build_int_cast(cmp_res, ctx.ctx().bool_type(), "ToBoolean")
        .expect("FATAL: LLVM failed to build_int_cast")
        .into()
}

pub fn models() -> HashMap<&'static str, LibClassModel> {
    let mut hashmap = HashMap::new();
    hashmap.insert(
        "Class",
        LibBlanketModel {
            id: LibClassId::Class,
            parent: LibClassId::Class,
            constructors: vec![],
            methods: HashMap::new(),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "AnyRef",
        LibObjectModel {
            id: LibClassId::AnyRef,
            parent: LibClassId::Class,
            init_impl: |ctx| {
                let func_type = ctx
                    .ctx()
                    .void_type()
                    .fn_type(&[ctx.ctx().ptr_type(AddressSpace::default()).into()], false);
                ctx.module().add_function("cls_AnyRef_init", func_type, None)
            },
            constructors: vec![(
                ParamsSignature::empty(),
                LibConsObjectImpl {
                    def_impl: |ctx| {
                        let func_type = ctx
                            .ctx()
                            .void_type()
                            .fn_type(&[ctx.ctx().ptr_type(AddressSpace::default()).into()], false);
                        ctx.module().add_function("cls_AnyRef_cons_args_", func_type, None)
                    },
                },
            )],
            methods: HashMap::from([(
                "ToString",
                vec![(
                    MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::String.into()),
                        overriding: None,
                    },
                    LibMethodObjectImpl {
                        def_impl: |llvm| {
                            let func_type = llvm.ctx().ptr_type(AddressSpace::default()).fn_type(
                                &[llvm.ctx().ptr_type(AddressSpace::default()).into()],
                                false,
                            );
                            llvm.module().add_function(
                                "cls_AnyRef_method_ToString_args_",
                                func_type,
                                None,
                            )
                        },
                    },
                )],
            )]),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "AnyValue",
        LibBlanketModel {
            id: LibClassId::AnyValue,
            parent: LibClassId::Class,
            constructors: vec![],
            methods: HashMap::new(),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "Integer",
        LibValueModel {
            id: LibClassId::Integer,
            parent: LibClassId::AnyValue,
            constructors: vec![
                (
                    ParamsSignature::new(vec![LibClassId::Integer.into()]),
                    LibConsValueImpl { call_impl: |ctx, args| *args.first().unwrap() },
                ),
                (
                    ParamsSignature::new(vec![LibClassId::Real.into()]),
                    LibConsValueImpl {
                        call_impl: |ctx, args| {
                            let f = args.first().unwrap();
                            float_to_int(ctx, f.into_float_value()).into()
                        },
                    },
                ),
            ],
            methods: HashMap::from([
                (
                    "ToReal",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                int_to_float(ctx, object.into_int_value()).into()
                            },
                        },
                    )],
                ),
                (
                    "ToBoolean",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(i, ctx.ctx().bool_type(), "ToBoolean")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "ToChar",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Char.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(i, ctx.ctx().i8_type(), "ToChar")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "UnaryMinus",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_neg(i, "UnaryMinus")
                                    .expect("FATAL: LLVM failed to build_int_neg")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Plus",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_int_add")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let i = object.into_int_value();
                                    let left = int_to_float(ctx, object.into_int_value());
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_float_add")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Minus",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_int_sub")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let i = object.into_int_value();
                                    let left = int_to_float(ctx, object.into_int_value());
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_float_sub")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Mult",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_int_mul")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let i = object.into_int_value();
                                    let left = int_to_float(ctx, object.into_int_value());
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_float_mul")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Div",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_signed_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_int_div")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = int_to_float(ctx, object.into_int_value());
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Rem",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_int_signed_rem(left, right, "Rem")
                                    .expect("FATAL: LLVM failed to build_int_rem")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Less",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SLT, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OLT,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                            },
                        ),
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SLE, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OLE,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Greater",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SGT, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OGT,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                            },
                        ),
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SGE, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OGE,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Equal",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::EQ, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OEQ,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                            },
                        ),
                    ],
                ),
            ]),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "Real",
        LibValueModel {
            id: LibClassId::Real,
            parent: LibClassId::AnyValue,
            constructors: vec![
                (
                    ParamsSignature::new(vec![LibClassId::Real.into()]),
                    LibConsValueImpl { call_impl: |ctx, args| *args.first().unwrap() },
                ),
                (
                    ParamsSignature::new(vec![LibClassId::Integer.into()]),
                    LibConsValueImpl {
                        call_impl: |ctx, args| {
                            let f = args.first().unwrap();
                            int_to_float(ctx, f.into_int_value()).into()
                        },
                    },
                ),
            ],
            methods: HashMap::from([
                (
                    "ToInteger",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                float_to_int(ctx, object.into_float_value()).into()
                            },
                        },
                    )],
                ),
                (
                    "UnaryMinus",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let f = object.into_float_value();
                                ctx.builder()
                                    .build_float_neg(f, "UnaryMinus")
                                    .expect("FATAL: LLVM failed to build_float_neg")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Plus",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_float_add")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_float_add")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Minus",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_float_sub")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_float_sub")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Mult",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_float_mul")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_float_mul")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Div",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Rem",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_float_value();
                                let i = args.first().expect("FATAL: no args").into_int_value();
                                let right = int_to_float(ctx, i);
                                ctx.builder()
                                    .build_float_rem(left, right, "Rem")
                                    .expect("FATAL: LLVM failed to build_float_rem")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Less",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OLT, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OLT, ctx, object, &[right.into()])
                                },
                            },
                        ),
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OLE, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OLE, ctx, object, &[right.into()])
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Greater",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OGT, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OGT, ctx, object, &[right.into()])
                                },
                            },
                        ),
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OGE, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OGE, ctx, object, &[right.into()])
                                },
                            },
                        ),
                    ],
                ),
                (
                    "Equal",
                    vec![
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OEQ, ctx, object, args)
                                },
                            },
                        ),
                        (
                            MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            LibMethodValueImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OEQ, ctx, object, &[right.into()])
                                },
                            },
                        ),
                    ],
                ),
            ]),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "Boolean",
        LibValueModel {
            id: LibClassId::Boolean,
            parent: LibClassId::AnyValue,
            constructors: vec![(
                ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                LibConsValueImpl { call_impl: |ctx, args| *args.first().unwrap() },
            )],
            methods: HashMap::from([
                (
                    "toInteger",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let b = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(b, ctx.ctx().i32_type(), "ToInteger")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Or",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_or(left, right, "Or")
                                    .expect("FATAL: LLVM failed to build_or")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "And",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_and(left, right, "And")
                                    .expect("FATAL: LLVM failed to build_and")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Xor",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_xor(left, right, "Xor")
                                    .expect("FATAL: LLVM failed to build_xor")
                                    .into()
                            },
                        },
                    )],
                ),
                (
                    "Not",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        LibMethodValueImpl {
                            call_impl: |ctx, object, args| {
                                let b = object.into_int_value();
                                ctx.builder()
                                    .build_not(b, "Not")
                                    .expect("FATAL: LLVM failed to build_not")
                                    .into()
                            },
                        },
                    )],
                ),
            ]),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "Array",
        LibObjectModel {
            id: LibClassId::Array,
            parent: LibClassId::AnyRef,
            init_impl: |ctx| todo!(),
            constructors: vec![(
                ParamsSignature::new(vec![LibClassId::Integer.into()]),
                LibConsObjectImpl { def_impl: |ctx| todo!() },
            )],
            methods: HashMap::from([
                (
                    "toList",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
                (
                    "Length",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
                (
                    "Get",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::AnyRef.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
                (
                    "Set",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![
                                LibClassId::Integer.into(),
                                LibClassId::AnyRef.into(),
                            ]),
                            return_type: None,
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
            ]),
            fields: HashMap::new(),
        }
        .into(),
    );

    hashmap.insert(
        "List",
        LibObjectModel {
            id: LibClassId::List,
            parent: LibClassId::AnyRef,
            init_impl: |ctx| todo!(),
            constructors: vec![
                (ParamsSignature::empty(), LibConsObjectImpl { def_impl: |ctx| todo!() }),
                (
                    ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                    LibConsObjectImpl { def_impl: |ctx| todo!() },
                ),
                (
                    ParamsSignature::new(vec![
                        LibClassId::AnyRef.into(),
                        LibClassId::Integer.into(),
                    ]),
                    LibConsObjectImpl { def_impl: |ctx| todo!() },
                ),
            ],
            methods: HashMap::from([
                (
                    "Append",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
                (
                    "Head",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::AnyRef.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
                (
                    "Tail",
                    vec![(
                        MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        LibMethodObjectImpl { def_impl: |ctx| todo!() },
                    )],
                ),
            ]),
            fields: HashMap::new(),
        }
        .into(),
    );

    //TODO char+string
    hashmap
}
