use llvm_plugin::inkwell::basic_block::BasicBlock;
use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::support::LLVMString;
use llvm_plugin::inkwell::values::{
    AnyValue, BasicValueEnum, FunctionValue, GlobalValue, InstructionOpcode, InstructionValue,
};
use llvm_plugin::{
    AnalysisKey, FunctionAnalysisManager, LlvmFunctionAnalysis, LlvmFunctionPass,
    LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PassBuilder, PipelineParsing,
    PreservedAnalyses,
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

    builder.add_module_analysis_registration_callback(|manager| {
        manager.register_pass(Ana1);
    });

    builder.add_function_analysis_registration_callback(|manager| {
        manager.register_pass(Ana2);
    });
}

struct Pass1;
impl LlvmModulePass for Pass1 {
    fn run_pass(&self, module: &mut Module, manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        let result = manager.get_cached_result::<Ana1>(module);
        assert!(result.is_none());

        let result = manager
            .get_result::<Ana1>(module)
            .as_ref()
            .expect("get_result");
        assert_eq!(result.to_string(), "[12 x i8] c\"hello world\\00\"");

        let result = manager.get_cached_result::<Ana1>(module);
        assert!(matches!(
            result.and_then(|res| res.as_ref()).map(|res| res.to_string()),
            Some(res) if res == "[12 x i8] c\"hello world\\00\""
        ));

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

static mut ANA1_CALL_COUNT: u32 = 0;

struct Ana1;
impl LlvmModuleAnalysis for Ana1 {
    type Result = Option<LLVMString>;

    fn run_analysis(&self, module: &Module, _manager: &ModuleAnalysisManager) -> Self::Result {
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

    fn id() -> AnalysisKey {
        1 as AnalysisKey
    }
}

static mut ANA2_CALL_COUNT: u32 = 0;

struct Ana2;
impl LlvmFunctionAnalysis for Ana2 {
    type Result = Option<InstructionOpcode>;

    fn run_analysis(
        &self,
        function: &FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> Self::Result {
        unsafe { ANA2_CALL_COUNT += 1 };
        function
            .get_last_basic_block()
            .and_then(BasicBlock::get_last_instruction)
            .map(InstructionValue::get_opcode)
    }

    fn id() -> AnalysisKey {
        2 as AnalysisKey
    }
}
