// See https://github.com/banach-space/llvm-tutor/blob/main/HelloWorld/HelloWorld.cpp
// for a more detailed explanation.

#[llvm_plugin::plugin(name = "HelloWorld", version = "0.1")]
mod plugin {
    use llvm_plugin::inkwell::values::FunctionValue;
    use llvm_plugin::{FunctionAnalysisManager, LlvmFunctionPass, PreservedAnalyses};

    #[derive(Default)]
    struct HelloWorldPass;

    #[pass(name = "hello-world")]
    impl LlvmFunctionPass for HelloWorldPass {
        fn run_pass(
            &self,
            function: &mut FunctionValue,
            _manager: &FunctionAnalysisManager,
        ) -> PreservedAnalyses {
            eprintln!("(llvm-tutor) Hello from: {:?}", function.get_name());
            eprintln!(
                "(llvm-tutor)   number of arguments: {}",
                function.count_params()
            );
            PreservedAnalyses::All
        }
    }
}
