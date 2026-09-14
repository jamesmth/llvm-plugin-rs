use llvm_plugin::inkwell::module::Module;
use llvm_plugin::inkwell::values::FunctionValue;
use llvm_plugin::{
    FunctionAnalysisManager, LlvmFunctionPass, LlvmModulePass, ModuleAnalysisManager,
    OptimizationLevel, PassBuilder, PreservedAnalyses,
};

#[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))]
use llvm_plugin::ThinOrFullLTOPhase;

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

    builder.add_vectorizer_start_ep_callback(|manager, opt| {
        assert!(matches!(opt, OptimizationLevel::O3));
        manager.add_pass(VectorizerStartPass);
    });

    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1",
        feature = "llvm19-1",
        feature = "llvm20-1",
        feature = "llvm21-1",
    ))]
    builder.add_pipeline_start_ep_callback(|manager, opt| {
        assert!(matches!(opt, OptimizationLevel::O3));
        manager.add_pass(PipelineStartPass);
    });

    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1",
        feature = "llvm19-1",
        feature = "llvm20-1",
        feature = "llvm21-1",
    ))]
    builder.add_pipeline_early_simplification_ep_callback(
        |manager, opt, #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))] thin_or_full| {
            assert!(matches!(opt, OptimizationLevel::O3));
            #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))]
            assert!(matches!(thin_or_full, ThinOrFullLTOPhase::None));
            manager.add_pass(PipelineEarlySimpPass);
        },
    );

    builder.add_optimizer_last_ep_callback(
        |manager, opt, #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))] thin_or_full| {
            assert!(matches!(opt, OptimizationLevel::O3));
            #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))]
            assert!(matches!(thin_or_full, ThinOrFullLTOPhase::None));
            manager.add_pass(OptimizerLastPass);
        },
    );

    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1",
        feature = "llvm19-1",
        feature = "llvm20-1",
        feature = "llvm21-1",
    ))]
    builder.add_optimizer_early_ep_callback(
        |manager, opt, #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))] thin_or_full| {
            assert!(matches!(opt, OptimizationLevel::O3));
            #[cfg(any(feature = "llvm20-1", feature = "llvm21-1"))]
            assert!(matches!(thin_or_full, ThinOrFullLTOPhase::None));
            manager.add_pass(OptimizerEarlyPass);
        },
    );
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

static mut VEC_START_PASS_CALLED: u32 = 0;

struct VectorizerStartPass;
impl LlvmFunctionPass for VectorizerStartPass {
    fn run_pass(
        &self,
        _function: &mut FunctionValue,
        _manager: &FunctionAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { VEC_START_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for VectorizerStartPass {
    fn drop(&mut self) {
        assert!(unsafe { VEC_START_PASS_CALLED } > 0);
    }
}

static mut PIPE_START_PASS_CALLED: u32 = 0;

struct PipelineStartPass;
impl LlvmModulePass for PipelineStartPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { PIPE_START_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for PipelineStartPass {
    fn drop(&mut self) {
        assert!(unsafe { PIPE_START_PASS_CALLED } > 0);
    }
}

static mut PIPE_EARLY_PASS_CALLED: u32 = 0;

struct PipelineEarlySimpPass;
impl LlvmModulePass for PipelineEarlySimpPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { PIPE_EARLY_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for PipelineEarlySimpPass {
    fn drop(&mut self) {
        assert!(unsafe { PIPE_EARLY_PASS_CALLED } > 0);
    }
}

static mut OPT_LAST_PASS_CALLED: u32 = 0;

struct OptimizerLastPass;
impl LlvmModulePass for OptimizerLastPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { OPT_LAST_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for OptimizerLastPass {
    fn drop(&mut self) {
        assert!(unsafe { OPT_LAST_PASS_CALLED } > 0);
    }
}

static mut OPT_EARLY_PASS_CALLED: u32 = 0;

struct OptimizerEarlyPass;
impl LlvmModulePass for OptimizerEarlyPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { OPT_EARLY_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for OptimizerEarlyPass {
    fn drop(&mut self) {
        assert!(unsafe { OPT_EARLY_PASS_CALLED } > 0);
    }
}
