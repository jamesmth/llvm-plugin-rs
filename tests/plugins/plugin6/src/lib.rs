use llvm_plugin::inkwell::values::FunctionValue;
use llvm_plugin::{
    FunctionAnalysisManager, LlvmFunctionPass, OptimizationLevel, PassBuilder, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_peephole_ep_callback(|manager, opt| {
        assert!(matches!(opt, OptimizationLevel::O3));
        manager.add_pass(Pass);
    });
}

static mut PASS_CALLED: u32 = 0;

struct Pass;
impl LlvmFunctionPass for Pass {
    fn run_pass(
        &self,
        _function: &mut FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        assert!(unsafe { PASS_CALLED } > 0);
    }
}
