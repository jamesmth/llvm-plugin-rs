#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
mod plugin {
    use llvm_plugin::inkwell::module::Module;
    use llvm_plugin::inkwell::support::LLVMString;
    use llvm_plugin::inkwell::values::{AnyValue, BasicValueEnum, GlobalValue};
    use llvm_plugin::{
        LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
    };

    #[derive(Default)]
    struct Pass1;

    #[pass(name = "mpass1")]
    impl LlvmModulePass for Pass1 {
        fn run_pass(
            &self,
            module: &mut Module,
            manager: &ModuleAnalysisManager,
        ) -> PreservedAnalyses {
            let result = manager
                .get_result::<Ana1>(module)
                .as_ref()
                .expect("get_result");
            assert_eq!(result.to_string(), "[13 x i8] c\"hello world\\0A\\00\"");
            PreservedAnalyses::None
        }
    }

    #[derive(Default)]
    struct Pass2;

    #[pass(name = "mpass2")]
    impl LlvmModulePass for Pass2 {
        fn run_pass(
            &self,
            module: &mut Module,
            manager: &ModuleAnalysisManager,
        ) -> PreservedAnalyses {
            let result = manager
                .get_result::<Ana1>(module)
                .as_ref()
                .expect("get_result");
            assert_eq!(result.to_string(), "[13 x i8] c\"hello world\\0A\\00\"");
            assert_eq!(unsafe { ANA1_CALL_COUNT }, 2);
            PreservedAnalyses::All
        }
    }

    #[derive(Default)]
    struct Ana1 {}
    static mut ANA1_CALL_COUNT: u32 = 0;

    #[analysis]
    impl LlvmModuleAnalysis for Ana1 {
        fn run_analysis(
            &self,
            module: &Module,
            _manager: &ModuleAnalysisManager,
        ) -> Option<LLVMString> {
            unsafe { ANA1_CALL_COUNT += 1 };
            module
                .get_first_global()
                .and_then(GlobalValue::get_initializer)
                .and_then(|init| match init {
                    BasicValueEnum::ArrayValue(v) => Some(v),
                    _ => None,
                })
                .map(|v| v.print_to_string())
        }
    }
}
