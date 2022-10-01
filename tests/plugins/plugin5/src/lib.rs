use llvm_plugin::inkwell::basic_block::BasicBlock;
use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::values::{FunctionValue, InstructionOpcode, InstructionValue};
use llvm_plugin::utils::FunctionIterator;
use llvm_plugin::{
    AnalysisKey, FunctionAnalysisManager, LlvmFunctionAnalysis, LlvmFunctionPass, LlvmModulePass,
    ModuleAnalysisManager, PassBuilder, PipelineParsing, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, pass_manager| {
        if name == "mpass" {
            pass_manager.add_pass(Pass1);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });

    builder.add_function_pipeline_parsing_callback(|name, pass_manager| {
        if name == "fpass" {
            pass_manager.add_pass(Pass2);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });

    builder.add_function_analysis_registration_callback(|manager| {
        manager.register_pass(Ana1);
    });
}

struct Pass1;
impl LlvmModulePass for Pass1 {
    fn run_pass(&self, module: &mut Module, manager: &ModuleAnalysisManager) -> PreservedAnalyses {
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

struct Pass2;
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

struct Ana1;
impl LlvmFunctionAnalysis for Ana1 {
    type Result = Option<InstructionOpcode>;

    fn run_analysis(
        &self,
        function: &FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> Self::Result {
        function
            .get_last_basic_block()
            .and_then(BasicBlock::get_last_instruction)
            .map(InstructionValue::get_opcode)
    }

    fn id() -> AnalysisKey {
        1 as AnalysisKey
    }
}
