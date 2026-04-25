use crate::{
    analyzer::{
        model::{ClassModel, MethodModel},
        registry::{MethodId, Registry, UserClassId},
    },
    codegen::{CodegenContext, GetValueType, LLVMCtx},
};

impl<'ctx, 'src> CodegenContext<'ctx, 'src> {
    fn codegen_method(
        &'ctx self,
        cls_id: &UserClassId,
        cls: &ClassModel,
        method_id: MethodId,
        method: &MethodModel,
    ) {
        let method_impl = self.impls.get(cls_id).methods.get_method(&method_id);
        let func = method_impl.func;

        let fst_entry = self.ctx().append_basic_block(func, "entry");
        self.builder().position_at_end(fst_entry);

        let llvm_vars: Registry<_, _> = method
            .body
            .body
            .vars
            .iter()
            // .zip(func.get_params())
            .map(|(var_id, var)| {
                let param_ptr = self
                    .builder()
                    .build_alloca(self.get_value_type(&var.var_type), &var_id.to_string())
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

        self.codegen_body(todo!(), &method.body.body);
        todo!()
    }
}
