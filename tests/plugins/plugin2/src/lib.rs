use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::support::LLVMString;
use llvm_plugin::inkwell::values::{AnyValue, BasicValueEnum, GlobalValue};
use llvm_plugin::{
    AnalysisKey, LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PassBuilder,
    PipelineParsing, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, pass_manager| {
        if name == "mpass1" {
            pass_manager.add_pass(Pass1);
            PipelineParsing::Parsed
        } else if name == "mpass2" {
            pass_manager.add_pass(Pass2);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });

    builder.add_module_analysis_registration_callback(|manager| {
        manager.register_pass(Ana1);
    });
}

struct Pass1;
impl LlvmModulePass for Pass1 {
    fn run_pass(&self, module: &mut Module, manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        let result = manager
            .get_result::<Ana1>(module)
            .as_ref()
            .expect("get_result");
        assert_eq!(result.to_string(), "[12 x i8] c\"hello world\\00\"");
        PreservedAnalyses::All
    }
}

struct Pass2;
impl LlvmModulePass for Pass2 {
    fn run_pass(&self, module: &mut Module, manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        let result = manager
            .get_result::<Ana1>(module)
            .as_ref()
            .expect("get_result");
        assert_eq!(result.to_string(), "[12 x i8] c\"hello world\\00\"");
        assert_eq!(unsafe { ANA1_CALL_COUNT }, 1);
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
