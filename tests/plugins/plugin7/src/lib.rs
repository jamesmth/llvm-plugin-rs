use llvm_plugin::inkwell::module::Module;
use llvm_plugin::{LlvmModulePass, ModuleAnalysisManager, PassBuilder, PreservedAnalyses};

#[llvm_plugin::plugin(name = "llvm_plugin", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1",
        feature = "llvm19-1",
        feature = "llvm20-1",
    ))]
    builder.add_full_lto_early_ep_callback(|manager, opt| {
        assert!(matches!(opt, llvm_plugin::OptimizationLevel::O3));
        manager.add_pass(FullLtoEarlyPass);
    });

    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-1",
        feature = "llvm19-1",
        feature = "llvm20-1",
    ))]
    builder.add_full_lto_last_ep_callback(|manager, opt| {
        assert!(matches!(opt, llvm_plugin::OptimizationLevel::O3));
        manager.add_pass(FullLtoLastPass);
    });

    let _ = builder;
}

static mut LTO_EARLY_PASS_CALLED: u32 = 0;

struct FullLtoEarlyPass;
impl LlvmModulePass for FullLtoEarlyPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { LTO_EARLY_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for FullLtoEarlyPass {
    fn drop(&mut self) {
        assert!(unsafe { LTO_EARLY_PASS_CALLED } > 0);
    }
}

static mut LTO_LAST_PASS_CALLED: u32 = 0;

struct FullLtoLastPass;
impl LlvmModulePass for FullLtoLastPass {
    fn run_pass(
        &self,
        _module: &mut Module,
        _manager: &ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        unsafe { LTO_LAST_PASS_CALLED += 1 };
        PreservedAnalyses::All
    }
}

impl Drop for FullLtoLastPass {
    fn drop(&mut self) {
        assert!(unsafe { LTO_LAST_PASS_CALLED } > 0);
    }
}
