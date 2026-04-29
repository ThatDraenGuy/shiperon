use std::collections::VecDeque;

use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    values::{
        AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue,
    },
};

use crate::{
    StdlibCtx,
    analyzer::{
        body::{AssignTarget, Body, BodyReturn, Stmt},
        expr::{CallExpr, Expr, ExprModel, PrimitiveExpr},
        field::{FieldExpr, FieldModel},
        registry::{ClassId, FieldId, Registry, VarId},
    },
    codegen::{ClassImpl, CodegenContext, GetFieldModels, GetFieldNameCtx, GetValueType, LLVMCtx},
};

type BodyScope<'ctx> = Registry<VarId, (ClassId, PointerValue<'ctx>)>;

pub struct ScopeStack<'ctx> {
    inner: VecDeque<BodyScope<'ctx>>,
    this_ptr: PointerValue<'ctx>,
    func: FunctionValue<'ctx>,
    vars_block: BasicBlock<'ctx>,
}
impl<'ctx> ScopeStack<'ctx> {
    pub fn new(
        scope: BodyScope<'ctx>,
        this_ptr: PointerValue<'ctx>,
        func: FunctionValue<'ctx>,
        vars_block: BasicBlock<'ctx>,
    ) -> Self {
        let mut inner = VecDeque::new();
        inner.push_back(scope);
        Self { inner, this_ptr, func, vars_block }
    }
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
    fn get_cls_impl(&self, cls_id: &ClassId) -> &ClassImpl<'ctx> {
        match cls_id {
            ClassId::User(user_class_id) => self.impls.get(user_class_id),
            ClassId::Lib(lib_class_id) => self.stdlib_impl.get(lib_class_id),
            ClassId::Invalid => unreachable!(),
        }
    }

    pub fn resolve_field(
        &self,
        object: BasicValueEnum<'ctx>,
        cls_id: &ClassId,
        field_id: &FieldId,
    ) -> PointerValue<'ctx> {
        let cls_impl = self.get_cls_impl(cls_id);
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
    pub fn alloc_body_vars(
        &self,
        body: &Body,
        vars_block: BasicBlock<'ctx>,
    ) -> Registry<VarId, (ClassId, PointerValue<'ctx>)> {
        self.builder().position_at_end(vars_block);
        body.vars
            .iter()
            .map(|(var_id, var)| {
                let param_ptr = self
                    .builder()
                    .build_alloca(self.get_value_type(&var.var_type), &format!("{var_id}_Alloc"))
                    .expect("FATAL: LLVM failed to build_alloca");
                (var_id, (var.var_type, param_ptr))
            })
            .collect()
    }

    pub fn codegen_body(&self, scopes: &mut ScopeStack<'ctx>, body: &Body) {
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
                Stmt::While { condition, body } => {
                    let condition_block =
                        self.ctx().append_basic_block(scopes.func, "while_condition");
                    let body_block = self.ctx().append_basic_block(scopes.func, "while_body");
                    let after_block = self.ctx().append_basic_block(scopes.func, "while_after");

                    self.builder()
                        .build_unconditional_branch(condition_block)
                        .expect("FATAL: LLVM failed to build_branch");

                    self.builder().position_at_end(condition_block);
                    let condition = self.codegen_expr(scopes, condition).into_int_value();
                    self.builder()
                        .build_conditional_branch(condition, body_block, after_block)
                        .expect("FATAL: LLVM failed to build_branch");

                    let body_vars = self.alloc_body_vars(body, scopes.vars_block);
                    self.builder().position_at_end(body_block);

                    scopes.enter(body_vars);
                    self.codegen_body(scopes, body);
                    scopes.exit();
                    self.builder()
                        .build_unconditional_branch(condition_block)
                        .expect("FATAL: LLVM failed to build_branch");

                    self.builder().position_at_end(after_block);
                },
                Stmt::If { condition, then_body, else_body } => {
                    let condition_block =
                        self.ctx().append_basic_block(scopes.func, "if_condition");
                    let then_block = self.ctx().append_basic_block(scopes.func, "then_body");
                    let else_block = if else_body.is_some() {
                        Some(self.ctx().append_basic_block(scopes.func, "else_body"))
                    } else {
                        None
                    };
                    let after_block = self.ctx().append_basic_block(scopes.func, "after_if");

                    self.builder()
                        .build_unconditional_branch(condition_block)
                        .expect("FATAL: LLVM failed to build_branch");
                    self.builder().position_at_end(condition_block);
                    let condition = self.codegen_expr(scopes, condition).into_int_value();
                    self.builder()
                        .build_conditional_branch(
                            condition,
                            then_block,
                            else_block.unwrap_or(after_block),
                        )
                        .expect("FATAL: LLVM failed to build_branch");

                    let then_vars = self.alloc_body_vars(then_body, scopes.vars_block);
                    self.builder().position_at_end(then_block);
                    scopes.enter(then_vars);
                    self.codegen_body(scopes, then_body);
                    scopes.exit();
                    self.builder()
                        .build_unconditional_branch(after_block)
                        .expect("FATAL: LLVM failed to build_branch");

                    if let Some(else_body) = else_body {
                        let else_vars = self.alloc_body_vars(else_body, scopes.vars_block);
                        self.builder().position_at_end(else_block.unwrap());
                        scopes.enter(else_vars);
                        self.codegen_body(scopes, else_body);
                        scopes.exit();
                        self.builder()
                            .build_unconditional_branch(after_block)
                            .expect("FATAL: LLVM failed to build_branch");
                    }
                    self.builder().position_at_end(after_block);
                },
                Stmt::Call(call) => {
                    self.codegen_call(scopes, call);
                },
                Stmt::Invalid => unreachable!(),
            }
        }
        match &body.return_expr {
            Some(BodyReturn::Value(expr)) => {
                let ret_val = self.codegen_expr(scopes, expr);
                self.builder()
                    .build_return(Some(&ret_val))
                    .expect("FATAL: LLVM failed to build_return");
            },
            Some(BodyReturn::Void) => {
                self.builder().build_return(None).expect("FATAL: LLVM failed to build_return");
            },
            Some(BodyReturn::Never) => {
                self.builder()
                    .build_unreachable()
                    .expect("FATAL: LLVM failed to build_unreachable");
            },
            None => {},
        };
    }

    fn codegen_primitive(&self, primitive: &PrimitiveExpr) -> BasicValueEnum<'ctx> {
        match primitive {
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
        }
    }

    pub fn codegen_field(&self, field: &FieldModel) -> BasicValueEnum<'ctx> {
        match &field.init_expr {
            FieldExpr::Primitive(primitive_expr) => self.codegen_primitive(primitive_expr),
            FieldExpr::Cons { class, cons, args } => {
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| self.codegen_field(arg))
                    // .map(BasicMetadataValueEnum::from)
                    .collect();

                match class {
                    ClassId::User(user_class_id) => {
                        let cons_impl = self.impls.get(user_class_id).constructors.get(cons);
                        let cons_func = cons_impl.func;

                        let meta_args: Vec<_> =
                            args.into_iter().map(BasicMetadataValueEnum::from).collect();
                        let call_res = self
                            .builder()
                            .build_call(cons_func, &meta_args, "cons")
                            .expect("FATAL: LLVM failed to build_call");
                        call_res.try_as_basic_value().unwrap_basic()
                    },
                    ClassId::Lib(lib_class_id) => {
                        (self.stdlib().cls_impl(lib_class_id).constructors.get(cons).call_impl)(
                            self, &args,
                        )
                    },
                    ClassId::Invalid => unreachable!(),
                }
            },
            FieldExpr::Invalid => unreachable!(),
        }
    }

    pub fn codegen_expr(
        &self,
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
            Expr::Primitive(primitive_expr) => self.codegen_primitive(primitive_expr),
            Expr::This => scopes.this_ptr.into(),
            Expr::ClassCast { expr, cls_id } => todo!(),
            Expr::Invalid => todo!(),
        }
    }

    pub fn codegen_assign_target(
        &self,
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
        &self,
        scopes: &mut ScopeStack<'ctx>,
        call: &CallExpr,
    ) -> AnyValueEnum<'ctx> {
        match call {
            CallExpr::Cons { class, cons, args } => {
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| self.codegen_expr(scopes, arg))
                    // .map(BasicMetadataValueEnum::from)
                    .collect();

                match class {
                    ClassId::User(user_class_id) => {
                        let cons_impl = self.impls.get(user_class_id).constructors.get(cons);
                        let cons_func = cons_impl.func;

                        let meta_args: Vec<_> =
                            args.into_iter().map(BasicMetadataValueEnum::from).collect();
                        let call_res = self
                            .builder()
                            .build_call(cons_func, &meta_args, "cons")
                            .expect("FATAL: LLVM failed to build_call");
                        call_res.as_any_value_enum()
                    },
                    ClassId::Lib(lib_class_id) => {
                        (self.stdlib().cls_impl(lib_class_id).constructors.get(cons).call_impl)(
                            self, &args,
                        )
                        .as_any_value_enum()
                    },
                    ClassId::Invalid => unreachable!(),
                }
            },
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
