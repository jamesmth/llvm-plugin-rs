use llvm_plugin::inkwell::module::Module;
use llvm_plugin::{
    LlvmModulePass, ModuleAnalysisManager, PassBuilder, PipelineParsing, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "pass_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, manager| {
        if name == "mpass" {
            manager.add_pass(Pass);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });
}

static mut CALL_COUNT: u32 = 0;

struct Pass;
impl LlvmModulePass for Pass {
    fn run_pass(&self, module: &mut Module, _manager: &ModuleAnalysisManager) -> PreservedAnalyses {
        if matches!(
            module.get_source_file_name().to_str(),
            Ok(s) if s.contains("build_script_build")
        ) {
            return PreservedAnalyses::All;
        }

        unsafe { CALL_COUNT += 1 };

        let path = if cfg!(target_os = "windows") {
            "C:\\rust.ll".to_owned()
        } else {
            let home = std::env::var("HOME").expect("missing HOME");
            home + "/rust.ll"
        };

        let _ = std::fs::write(path, unsafe { CALL_COUNT }.to_string().as_bytes());

        PreservedAnalyses::All
    }
}
