use llvm_plugin::inkwell::values::FunctionValue;
use llvm_plugin::{
    FunctionAnalysisManager, LlvmFunctionPass, OptimizationLevel, PassBuilder, PreservedAnalyses,
};

#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_peephole_ep_callback(|manager, opt| {
        assert!(matches!(opt, OptimizationLevel::O3));
        manager.add_pass(PeepholePass);
    });

    builder.add_scalar_optimizer_late_ep_callback(|manager, opt| {
        assert!(matches!(opt, OptimizationLevel::O3));
        manager.add_pass(OptimizerLatePass);
    });
}

static mut PEEPHOLE_PASS_CALLED: u32 = 0;

struct PeepholePass;
impl LlvmFunctionPass for PeepholePass {
    fn run_pass(
        &self,
        _function: &mut FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { PEEPHOLE_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for PeepholePass {
    fn drop(&mut self) {
        assert!(unsafe { PEEPHOLE_PASS_CALLED } > 0);
    }
}

static mut OPT_LATE_PASS_CALLED: u32 = 0;

struct OptimizerLatePass;
impl LlvmFunctionPass for OptimizerLatePass {
    fn run_pass(
        &self,
        _function: &mut FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { OPT_LATE_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for OptimizerLatePass {
    fn drop(&mut self) {
        assert!(unsafe { OPT_LATE_PASS_CALLED } > 0);
    }
}
