#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
mod plugin {
    use llvm_plugin::inkwell::basic_block::BasicBlock;
    use llvm_plugin::inkwell::module::Module;
    use llvm_plugin::inkwell::support::LLVMString;
    use llvm_plugin::inkwell::values::{
        AnyValue, BasicValueEnum, FunctionValue, GlobalValue, InstructionOpcode, InstructionValue,
    };
    use llvm_plugin::{
        FunctionAnalysisManager, LlvmFunctionAnalysis, LlvmFunctionPass, LlvmModuleAnalysis,
        LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
    };

    #[derive(Default)]
    struct Pass1;

    #[pass(name = "mpass")]
    impl LlvmModulePass for Pass1 {
        fn run_pass(
            &self,
            module: &mut Module,
            manager: &ModuleAnalysisManager,
        ) -> PreservedAnalyses {
            let result = manager.get_cached_result::<Ana1>(module);
            assert!(result.is_none());

            let result = manager
                .get_result::<Ana1>(module)
                .as_ref()
                .expect("get_result");
            assert_eq!(result.to_string(), "[13 x i8] c\"hello world\\0A\\00\"");

            let result = manager.get_cached_result::<Ana1>(module);
            assert!(matches!(
                result.and_then(|res| res.as_ref()).map(|res| res.to_string()),
                Some(res) if res == "[13 x i8] c\"hello world\\0A\\00\""
            ));

            PreservedAnalyses::All
        }
    }

    #[derive(Default)]
    struct Pass2;

    #[pass(name = "fpass")]
    impl LlvmFunctionPass for Pass2 {
        fn run_pass(
            &self,
            function: &mut FunctionValue,
            manager: &FunctionAnalysisManager,
        ) -> PreservedAnalyses {
            let result = manager.get_cached_result::<Ana2>(function);
            assert!(result.is_none());

            let result = manager.get_result::<Ana2>(function).expect("get_result");
            assert_eq!(result, InstructionOpcode::Return);

            let result = manager.get_cached_result::<Ana2>(function);
            assert!(matches!(
                result.and_then(|res| res.as_ref()),
                Some(InstructionOpcode::Return)
            ));

            assert_eq!(unsafe { ANA1_CALL_COUNT }, 1);
            assert_eq!(unsafe { ANA2_CALL_COUNT }, 1);
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

    #[derive(Default)]
    struct Ana2 {}
    static mut ANA2_CALL_COUNT: u32 = 0;

    #[analysis]
    impl LlvmFunctionAnalysis for Ana2 {
        fn run_analysis(
            &self,
            function: &FunctionValue,
            _manager: &FunctionAnalysisManager,
        ) -> Option<InstructionOpcode> {
            unsafe { ANA2_CALL_COUNT += 1 };
            function
                .get_last_basic_block()
                .and_then(BasicBlock::get_last_instruction)
                .map(InstructionValue::get_opcode)
        }
    }
}
