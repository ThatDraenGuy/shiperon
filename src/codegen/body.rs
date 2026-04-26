use std::collections::{LinkedList, VecDeque};

use inkwell::{
    AddressSpace,
    types::BasicMetadataTypeEnum,
    values::{AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, PointerValue},
};

use crate::{
    StdlibCtx,
    analyzer::{
        body::{AssignTarget, Body, Stmt},
        expr::{CallExpr, Expr, ExprModel, PrimitiveExpr},
        field::FieldModel,
        model::ClassModelCtx,
        registry::{ClassId, FieldId, Registry, UserClassId, VarId},
    },
    codegen::{CodegenContext, GetFieldModels, GetFieldNameCtx, GetValueType, LLVMCtx},
};

type BodyScope<'ctx> = Registry<VarId, (ClassId, PointerValue<'ctx>)>;

pub struct ScopeStack<'ctx> {
    inner: VecDeque<BodyScope<'ctx>>,
}
impl<'ctx> ScopeStack<'ctx> {
    fn curr(&self) -> &BodyScope<'ctx> {
        self.inner.back().unwrap()
    }
    fn get(&self, offset: usize) -> &BodyScope<'ctx> {
        self.inner.get(self.inner.len() - 1 - offset).unwrap()
    }
    fn enter(&mut self, scope: BodyScope<'ctx>) {
        self.inner.push_back(scope);
    }
    fn exit(&mut self) -> BodyScope<'ctx> {
        self.inner.pop_back().unwrap()
    }
}

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    // pub fn get_field_model(&self, cls_id: &ClassId, field_id: &FieldId) -> &FieldModel {
    //     match cls_id {
    //         ClassId::User(user_class_id) => {
    //             self.ast.cls_models().get(user_class_id).fields.get(field_id)
    //         },
    //         ClassId::Lib(lib_class_id) => {
    //             self.stdlib().cls_fields(lib_class_id).registry.get(field_id)
    //         },
    //         ClassId::Invalid => unreachable!(),
    //     }
    // }

    pub fn resolve_field(
        &self,
        object: BasicValueEnum<'ctx>,
        cls_id: &ClassId,
        field_id: &FieldId,
    ) -> PointerValue<'ctx> {
        let cls_impl = self.impls.get(todo!());
        let struct_type = cls_impl.object_type;
        let obj_ptr = object.into_pointer_value(); //TODO rework
        let field_impl = cls_impl.fields.get(field_id);

        self.builder()
            .build_struct_gep(
                struct_type,
                obj_ptr,
                field_impl.struct_offset,
                self.field_name(cls_id, field_id),
            )
            .expect("FATAL: LLVM failed to build_struct_gep")
    }
    pub fn codegen_body(&'ctx self, scopes: &mut ScopeStack<'ctx>, body: &Body) {
        for stmt in &body.stmts {
            match stmt {
                Stmt::VarDef { id, init_expr } => {
                    let value = self.codegen_expr(scopes, init_expr);
                    let (_var_type, ptr) = scopes.curr().get(id); //var def always refers to current var scope
                    self.builder()
                        .build_store(*ptr, value)
                        .expect("FATAL: LLVM failed to build_store");
                },
                Stmt::Assign(assign_target, expr) => {
                    let value = self.codegen_expr(scopes, expr);
                    let target_ptr = self.codegen_assign_target(scopes, assign_target);
                    self.builder()
                        .build_store(target_ptr, value)
                        .expect("FATAL: LLVM failed to build_store");
                },
                Stmt::While { condition, body } => todo!(),
                Stmt::If { condition, then_body, else_body } => todo!(),
                Stmt::Call(call) => {
                    self.codegen_call(scopes, call);
                },
                Stmt::Invalid => unreachable!(),
            }
        }
    }

    pub fn codegen_expr(
        &'ctx self,
        scopes: &mut ScopeStack<'ctx>,
        expr: &ExprModel,
    ) -> BasicValueEnum<'ctx> {
        match &expr.expr {
            Expr::Variable(var_id, offset) => {
                let (var_type, ptr) = scopes.get(*offset).get(var_id);
                let load_type = self.get_value_type(var_type);
                self.builder()
                    .build_load(load_type, *ptr, &var_id.to_string())
                    .expect("FATAL: LLVM failed to build_load")
            },
            Expr::FieldRead { expr, field, owner_cls } => {
                let source = self.codegen_expr(scopes, expr);
                let field_ptr = self.resolve_field(source, owner_cls, field);
                let field_type = self.field_models(owner_cls).get(field).field_type;
                self.builder()
                    .build_load(
                        self.get_value_type(&field_type),
                        field_ptr,
                        self.field_name(owner_cls, field),
                    )
                    .expect("FATAL: LLVM failed to build_load")
            },
            Expr::Call(call_expr) => {
                let any_res = self.codegen_call(scopes, call_expr);
                BasicValueEnum::try_from(any_res).expect("FATAL: non basic value call used as expr")
            },
            Expr::Primitive(primitive_expr) => match primitive_expr {
                PrimitiveExpr::Integer(i) => {
                    let int_type = self.ctx().i32_type();
                    int_type.const_int(*i as u64, true).into()
                },
                PrimitiveExpr::Real(r) => {
                    let float_type = self.ctx().f32_type();
                    float_type.const_float((*r).into()).into()
                },
                PrimitiveExpr::String(s) => {
                    let string = self.ctx().const_string(s.as_bytes(), true);
                    string.into()
                },
                PrimitiveExpr::Char(c) => {
                    let char_type = self.ctx().i8_type();
                    char_type.const_int(*c as u64, false).into()
                },
            },
            Expr::This => todo!(),
            Expr::ClassCast { expr, cls_id } => todo!(),
            Expr::Invalid => todo!(),
        }
    }

    pub fn codegen_assign_target(
        &'ctx self,
        scopes: &mut ScopeStack<'ctx>,
        target: &AssignTarget,
    ) -> PointerValue<'ctx> {
        match target {
            AssignTarget::Var(var_id, offset) => {
                let (_var_type, ptr) = scopes.get(*offset).get(var_id);
                *ptr
            },
            AssignTarget::Field(expr, owner_cls, field) => {
                let source = self.codegen_expr(scopes, expr);

                self.resolve_field(source, owner_cls, field)
            },
            AssignTarget::Invalid => unreachable!(),
        }
    }

    pub fn codegen_call(
        &'ctx self,
        scopes: &mut ScopeStack<'ctx>,
        call: &CallExpr,
    ) -> AnyValueEnum<'ctx> {
        match call {
            CallExpr::Cons { class, cons, args } => todo!(),
            CallExpr::Method { object, class, method, args } => {
                let args: Vec<_> = args.iter().map(|arg| self.codegen_expr(scopes, arg)).collect();
                let object = self.codegen_expr(scopes, object);

                match class {
                    ClassId::User(user_class_id) => {
                        let cls_impl = self.impls.get(user_class_id);
                        let method_impl = cls_impl.methods.get_method(method);

                        let vtable_ptr = self
                            .builder()
                            .build_load(
                                self.ctx().ptr_type(AddressSpace::default()),
                                object.into_pointer_value(),
                                "vtable_ptr",
                            )
                            .expect("FATAL: LLVM failed to build_load");
                        //SAFETY: vtable should be safe (:
                        let method_ptr_ptr = unsafe {
                            self.builder()
                                .build_gep(
                                    self.ctx().ptr_type(AddressSpace::default()),
                                    vtable_ptr.into_pointer_value(),
                                    &[self
                                        .ctx()
                                        .i32_type()
                                        .const_int(method_impl.vtable_offset, false)],
                                    "method_ptr_ptr",
                                )
                                .expect("FATAL: LLVM failed to build_gep")
                        };
                        let method_ptr = self
                            .builder()
                            .build_load(
                                self.ctx().ptr_type(AddressSpace::default()),
                                method_ptr_ptr,
                                "method_ptr",
                            )
                            .expect("FATAL: LLVM failed to build_load");

                        let meta_args: Vec<_> =
                            args.into_iter().map(BasicMetadataValueEnum::from).collect();
                        let res = self
                            .builder()
                            .build_indirect_call(
                                method_impl.func.get_type(),
                                method_ptr.into_pointer_value(),
                                &meta_args,
                                "call",
                            )
                            .expect("FATAL: LLVM failed to build_inderect_call");
                        res.as_any_value_enum()
                    },
                    ClassId::Lib(lib_class_id) => {
                        (self.stdlib().cls_impl(lib_class_id).methods.get_method(method).call_impl)(
                            self, object, &args,
                        )
                    },
                    ClassId::Invalid => unreachable!(),
                }
            },
            CallExpr::Invalid => unreachable!(),
        }
    }
}
