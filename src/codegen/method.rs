use crate::{
    analyzer::{
        model::MethodModel,
        registry::{MethodId, Registry, UserClassId},
    },
    codegen::{CodegenContext, GetValueType, LLVMCtx, body::ScopeStack},
};

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    pub fn codegen_method(&self, cls_id: &UserClassId, method_id: MethodId, method: &MethodModel) {
        let method_impl = self.impls.get(cls_id).methods.get_method(&method_id);
        let func = method_impl.func;

        if !method.body.body.vars.is_empty() {
            let vars_entry = self.ctx().append_basic_block(func, "vars");
            self.builder().position_at_end(vars_entry);
        }

        let llvm_vars: Registry<_, _> = method
            .body
            .body
            .vars
            .iter()
            // .zip(func.get_params())
            .map(|(var_id, var)| {
                let param_ptr = self
                    .builder()
                    .build_alloca(self.get_value_type(&var.var_type), &format!("{var_id}_Alloc"))
                    .expect("FATAL: LLVM failed to build_alloca");
                (var_id, (var.var_type, param_ptr))
            })
            .collect();

        // store all args into their variables
        for ((_var_id, (_var_type, param_ptr)), llvm_param) in
            llvm_vars.iter().zip(func.get_params().into_iter().skip(1))
        //skip self ptr
        {
            self.builder()
                .build_store(*param_ptr, llvm_param)
                .expect("FATAL: LLVM failed to build_store");
        }

        let body_entry = self.ctx().append_basic_block(func, "body");

        if !method.body.body.vars.is_empty() {
            self.builder()
                .build_unconditional_branch(body_entry)
                .expect("FATAL: LLVM failed to build_branch");
        }
        self.builder().position_at_end(body_entry);

        let mut scopes = ScopeStack::new_method(
            llvm_vars,
            func.get_params().first().unwrap().into_pointer_value(),
        );
        self.codegen_body(&mut scopes, &method.body.body);
        if method.body.body.return_expr.is_none() {
            self.builder().build_return(None).expect("FATAL: LLVM failed to build return");
        }
    }
}
