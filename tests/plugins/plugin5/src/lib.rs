#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
mod plugin {
    use llvm_plugin::inkwell::basic_block::BasicBlock;
    use llvm_plugin::inkwell::module::Module;
    use llvm_plugin::inkwell::values::{FunctionValue, InstructionOpcode, InstructionValue};
    use llvm_plugin::utils::FunctionIterator;
    use llvm_plugin::{
        FunctionAnalysisManager, LlvmFunctionAnalysis, LlvmFunctionPass, LlvmModulePass,
        ModuleAnalysisManager, PreservedAnalyses,
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
            let manager = manager
                .get_function_analysis_manager_proxy(&module)
                .get_manager();

            for function in FunctionIterator::new(module) {
                let result = manager.get_cached_result::<Ana1>(&function);
                assert!(result.is_none());

                let result = manager.get_result::<Ana1>(&function);
                match function.get_name().to_bytes() {
                    b"main" => assert!(matches!(result, Some(InstructionOpcode::Return))),
                    b"puts" => assert!(matches!(result, None)),
                    _ => continue,
                }
            }

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
            let result = manager.get_cached_result::<Ana1>(&function);
            match function.get_name().to_bytes() {
                b"main" => assert!(matches!(
                    result.and_then(|res| res.as_ref()),
                    Some(InstructionOpcode::Return)
                )),
                b"puts" => assert!(matches!(result, Some(None))),
                _ => (),
            }

            PreservedAnalyses::All
        }
    }

    #[derive(Default)]
    struct Ana1;

    #[analysis]
    impl LlvmFunctionAnalysis for Ana1 {
        fn run_analysis(
            &self,
            function: &FunctionValue,
            _manager: &FunctionAnalysisManager,
        ) -> Option<InstructionOpcode> {
            function
                .get_last_basic_block()
                .and_then(BasicBlock::get_last_instruction)
                .map(InstructionValue::get_opcode)
        }
    }
}
