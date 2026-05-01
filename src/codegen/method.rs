use crate::{
    analyzer::{
        model::{ConsModel, MethodModel},
        registry::{ConsId, MethodId, UserClassId},
    },
    codegen::{CodegenContext, LLVMCtx, body::ScopeStack, clsimpl::ClassImpl},
};

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    pub fn codegen_cons(&self, cls_id: &UserClassId, cons_id: ConsId, cons: &ConsModel) {
        let cls_impl = self.impls.get(cls_id).unwrap_object_ref();
        let cons_impl = cls_impl.constructors.get(&cons_id);
        let func = cons_impl.func;

        let vars_block = self.ctx().append_basic_block(func, "vars");
        let llvm_vars = self.alloc_body_vars(&cons.body.body, vars_block);

        // store all args into their variables
        for ((_var_id, (_var_type, param_ptr)), llvm_param) in
            llvm_vars.iter().zip(func.get_params())
        //skip self ptr
        {
            self.builder()
                .build_store(*param_ptr, llvm_param)
                .expect("FATAL: LLVM failed to build_store");
        }

        let body_block = self.ctx().append_basic_block(func, "body");
        self.builder().position_at_end(body_block);

        let this_ptr = self
            .builder()
            .build_call(
                self.stdlib_impl.malloc,
                &[cls_impl.object_type.size_of().unwrap().into()],
                "malloc_call",
            )
            .expect("FATAL: LLVM failed to build_call")
            .try_as_basic_value()
            .unwrap_basic()
            .into_pointer_value();

        self.builder()
            .build_store(this_ptr, cls_impl.vtable.as_pointer_value())
            .expect("FATAL: LLVM failed to build_store");

        self.builder()
            .build_call(cls_impl.init_func, &[this_ptr.into()], "init_call")
            .expect("FATAL: LLVM failed to build_call");

        let mut scopes = ScopeStack::new(llvm_vars, this_ptr, func, vars_block);
        self.codegen_body(&mut scopes, &cons.body.body);

        self.builder().build_return(Some(&this_ptr)).expect("FATAL: LLVM failed to build_return");

        self.builder().position_at_end(vars_block);
        self.builder()
            .build_unconditional_branch(body_block)
            .expect("FATAL: LLVM failed to build_branch");
    }

    pub fn codegen_method(&self, cls_id: &UserClassId, method_id: MethodId, method: &MethodModel) {
        let cls_impl = self.impls.get(cls_id).unwrap_object_ref();
        let method_impl = cls_impl.methods.get_method(&method_id);
        let func = method_impl.func;

        let vars_block = self.ctx().append_basic_block(func, "vars");
        let llvm_vars = self.alloc_body_vars(&method.body.body, vars_block);

        // store all args into their variables
        for ((_var_id, (_var_type, param_ptr)), llvm_param) in
            llvm_vars.iter().zip(func.get_params().into_iter().skip(1))
        //skip self ptr
        {
            self.builder()
                .build_store(*param_ptr, llvm_param)
                .expect("FATAL: LLVM failed to build_store");
        }

        let body_block = self.ctx().append_basic_block(func, "body");
        self.builder().position_at_end(body_block);

        let mut scopes = ScopeStack::new(
            llvm_vars,
            func.get_params().first().unwrap().into_pointer_value(),
            func,
            vars_block,
        );
        self.codegen_body(&mut scopes, &method.body.body);
        if method.body.body.return_expr.is_none() {
            self.builder().build_return(None).expect("FATAL: LLVM failed to build return");
        }
        self.builder().position_at_end(vars_block);
        self.builder()
            .build_unconditional_branch(body_block)
            .expect("FATAL: LLVM failed to build_branch");
    }
}
