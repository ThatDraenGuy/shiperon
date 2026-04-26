use std::collections::HashMap;

use inkwell::{
    AddressSpace, FloatPredicate, IntPredicate,
    values::{AnyValueEnum, BasicValueEnum, FloatValue, IntValue},
};

use crate::{
    analyzer::{
        field::FieldModel,
        registry::LibClassId,
        signature::{MethodSignature, ParamsSignature},
    },
    codegen::{CodegenContext, LLVMCtx},
    stdlib::LibMethodImpl,
};

pub struct LibMethodModel {
    pub signature: MethodSignature,
    pub method_impl: LibMethodImpl,
}

pub struct LibClassModel {
    pub id: LibClassId,
    pub parent: LibClassId,
    pub constructors: Vec<ParamsSignature>,
    pub methods: HashMap<&'static str, Vec<LibMethodModel>>,
    pub fields: HashMap<&'static str, FieldModel>,
}

fn int_compare<'ctx, 'src>(
    predicate: IntPredicate,
    ctx: &'ctx CodegenContext<'ctx, 'src>,
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

fn int_to_float<'ctx, 'src>(
    ctx: &'ctx CodegenContext<'ctx, 'src>,
    i: IntValue<'ctx>,
) -> FloatValue<'ctx> {
    ctx.builder()
        .build_signed_int_to_float(i, ctx.ctx().f32_type(), "ToReal")
        .expect("FATAL: LLVM failed to build_sitf")
}

fn float_to_int<'ctx, 'src>(
    ctx: &'ctx CodegenContext<'ctx, 'src>,
    f: FloatValue<'ctx>,
) -> IntValue<'ctx> {
    ctx.builder()
        .build_float_to_signed_int(f, ctx.ctx().i32_type(), "ToInteger")
        .expect("FATAL: LLVM failed to build_ftsi")
}

fn float_compare<'ctx, 'src>(
    predicate: FloatPredicate,
    ctx: &'ctx CodegenContext<'ctx, 'src>,
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
        LibClassModel {
            id: LibClassId::Class,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::new(vec![])],
            methods: HashMap::new(),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "AnyRef",
        LibClassModel {
            id: LibClassId::AnyRef,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([(
                "ToString",
                vec![LibMethodModel {
                    signature: MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::String.into()),
                        overriding: None,
                    },
                    method_impl: LibMethodImpl {
                        call_impl: |ctx, object, args| todo!(),
                        def_impl: |llvm| {
                            let func_type = llvm.ctx().ptr_type(AddressSpace::default()).fn_type(
                                &[llvm.ctx().ptr_type(AddressSpace::default()).into()],
                                false,
                            );
                            Some(llvm.module().add_function(
                                "cls_AnyRef_method_ToString_args_",
                                func_type,
                                None,
                            ))
                        },
                    },
                }],
            )]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "AnyValue",
        LibClassModel {
            id: LibClassId::AnyValue,
            parent: LibClassId::Class,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([(
                "ToString",
                vec![LibMethodModel {
                    signature: MethodSignature {
                        params: ParamsSignature::empty(),
                        return_type: Some(LibClassId::String.into()),
                        overriding: None,
                    },
                    method_impl: LibMethodImpl {
                        call_impl: |ctx, object, args| todo!(),
                        def_impl: |ctx| None,
                    },
                }],
            )]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Integer",
        LibClassModel {
            id: LibClassId::Integer,
            parent: LibClassId::AnyValue,
            constructors: vec![
                ParamsSignature::new(vec![LibClassId::Integer.into()]),
                ParamsSignature::new(vec![LibClassId::Real.into()]),
            ],
            methods: HashMap::from([
                (
                    "ToReal",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                int_to_float(ctx, object.into_int_value()).into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "ToBoolean",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(i, ctx.ctx().bool_type(), "ToBoolean")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "ToChar",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Char.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(i, ctx.ctx().i8_type(), "ToChar")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "UnaryMinus",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let i = object.into_int_value();
                                ctx.builder()
                                    .build_int_neg(i, "UnaryMinus")
                                    .expect("FATAL: LLVM failed to build_int_neg")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Plus",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_int_add")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
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
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Minus",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_int_sub")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
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
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Mult",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_int_mul")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
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
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Div",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Integer.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_int_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_int_value();
                                    ctx.builder()
                                        .build_int_signed_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_int_div")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = int_to_float(ctx, object.into_int_value());
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Rem",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_int_signed_rem(left, right, "Rem")
                                    .expect("FATAL: LLVM failed to build_int_rem")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Less",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SLT, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OLT,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SLE, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OLE,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Greater",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SGT, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OGT,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::SGE, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OGE,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Equal",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    int_compare(IntPredicate::EQ, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(
                                        FloatPredicate::OEQ,
                                        ctx,
                                        int_to_float(ctx, object.into_int_value()).into(),
                                        args,
                                    )
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Real",
        LibClassModel {
            id: LibClassId::Real,
            parent: LibClassId::AnyValue,
            constructors: vec![
                ParamsSignature::new(vec![LibClassId::Real.into()]),
                ParamsSignature::new(vec![LibClassId::Integer.into()]),
            ],
            methods: HashMap::from([
                (
                    "ToInteger",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                float_to_int(ctx, object.into_float_value()).into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "UnaryMinus",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let f = object.into_float_value();
                                ctx.builder()
                                    .build_float_neg(f, "UnaryMinus")
                                    .expect("FATAL: LLVM failed to build_float_neg")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Plus",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_float_add")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_add(left, right, "Plus")
                                        .expect("FATAL: LLVM failed to build_float_add")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Minus",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_float_sub")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_sub(left, right, "Minus")
                                        .expect("FATAL: LLVM failed to build_float_sub")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Mult",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_float_mul")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_mul(left, right, "Mult")
                                        .expect("FATAL: LLVM failed to build_float_mul")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Div",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let right =
                                        args.first().expect("FATAL: no args").into_float_value();
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Real.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let left = object.into_float_value();
                                    let i = args.first().expect("FATAL: no args").into_int_value();
                                    let right = int_to_float(ctx, i);
                                    ctx.builder()
                                        .build_float_div(left, right, "Div")
                                        .expect("FATAL: LLVM failed to build_float_div")
                                        .into()
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Rem",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::Real.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_float_value();
                                let i = args.first().expect("FATAL: no args").into_int_value();
                                let right = int_to_float(ctx, i);
                                ctx.builder()
                                    .build_float_rem(left, right, "Rem")
                                    .expect("FATAL: LLVM failed to build_float_rem")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Less",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OLT, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OLT, ctx, object, &[right.into()])
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "LessEqual",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OLE, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OLE, ctx, object, &[right.into()])
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Greater",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OGT, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OGT, ctx, object, &[right.into()])
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "GreaterEqual",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OGE, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OGE, ctx, object, &[right.into()])
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
                (
                    "Equal",
                    vec![
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Real.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    float_compare(FloatPredicate::OEQ, ctx, object, args)
                                },
                                def_impl: |ctx| None,
                            },
                        },
                        LibMethodModel {
                            signature: MethodSignature {
                                params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                                return_type: Some(LibClassId::Boolean.into()),
                                overriding: None,
                            },
                            method_impl: LibMethodImpl {
                                call_impl: |ctx, object, args| {
                                    let right = int_to_float(
                                        ctx,
                                        args.first().expect("FATAL: no args").into_int_value(),
                                    );
                                    float_compare(FloatPredicate::OEQ, ctx, object, &[right.into()])
                                },
                                def_impl: |ctx| None,
                            },
                        },
                    ],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Boolean",
        LibClassModel {
            id: LibClassId::Boolean,
            parent: LibClassId::AnyValue,
            constructors: vec![ParamsSignature::empty()],
            methods: HashMap::from([
                (
                    "toInteger",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let b = object.into_int_value();
                                ctx.builder()
                                    .build_int_cast(b, ctx.ctx().i32_type(), "ToInteger")
                                    .expect("FATAL: LLVM failed to build_int_cast")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Or",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_or(left, right, "Or")
                                    .expect("FATAL: LLVM failed to build_or")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "And",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_and(left, right, "And")
                                    .expect("FATAL: LLVM failed to build_and")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Xor",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Boolean.into()]),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let left = object.into_int_value();
                                let right = args.first().expect("FATAL: no args").into_int_value();
                                ctx.builder()
                                    .build_xor(left, right, "Xor")
                                    .expect("FATAL: LLVM failed to build_xor")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
                (
                    "Not",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Boolean.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| {
                                let b = object.into_int_value();
                                ctx.builder()
                                    .build_not(b, "Not")
                                    .expect("FATAL: LLVM failed to build_not")
                                    .into()
                            },
                            def_impl: |ctx| None,
                        },
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "Array",
        LibClassModel {
            id: LibClassId::Array,
            parent: LibClassId::AnyRef,
            constructors: vec![ParamsSignature::new(vec![LibClassId::Integer.into()])],
            methods: HashMap::from([
                (
                    "toList",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
                (
                    "Length",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::Integer.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
                (
                    "Get",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::Integer.into()]),
                            return_type: Some(LibClassId::AnyRef.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
                (
                    "Set",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![
                                LibClassId::Integer.into(),
                                LibClassId::AnyRef.into(),
                            ]),
                            return_type: None,
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    hashmap.insert(
        "List",
        LibClassModel {
            id: LibClassId::List,
            parent: LibClassId::AnyRef,
            constructors: vec![
                ParamsSignature::empty(),
                ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                ParamsSignature::new(vec![LibClassId::AnyRef.into(), LibClassId::Integer.into()]),
            ],
            methods: HashMap::from([
                (
                    "Append",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::new(vec![LibClassId::AnyRef.into()]),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
                (
                    "Head",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::AnyRef.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
                (
                    "Tail",
                    vec![LibMethodModel {
                        signature: MethodSignature {
                            params: ParamsSignature::empty(),
                            return_type: Some(LibClassId::List.into()),
                            overriding: None,
                        },
                        method_impl: LibMethodImpl {
                            call_impl: |ctx, object, args| todo!(),
                            def_impl: |ctx| todo!(),
                        },
                    }],
                ),
            ]),
            fields: HashMap::new(),
        },
    );

    //TODO char+string
    hashmap
}
