#include <mutex>

#include <llvm/IR/PassManager.h>
#include <llvm/Passes/PassBuilder.h>
#include <llvm/Passes/PassPlugin.h>

#include "analysis.hh"
#include "common.hh"
#include "pass.hh"

extern "C" {
auto getFunctionAnalysisManagerModuleProxy(llvm::ModuleAnalysisManager &AM,
                                           llvm::Module &Module) -> void * {
  auto &FAMProxy =
      AM.getResult<llvm::FunctionAnalysisManagerModuleProxy>(Module);
  return static_cast<void *>(&FAMProxy);
}

auto getFunctionAnalysisManager(
    llvm::FunctionAnalysisManagerModuleProxy::Result &FAMProxy) -> void * {
  auto &FAM = FAMProxy.getManager();
  return static_cast<void *>(&FAM);
}

auto getModuleAnalysisResult(llvm::ModuleAnalysisManager &AM,
                             llvm::AnalysisKey *Key, llvm::Module &Module)
    -> void * {
  const auto Lock = std::lock_guard{Analysis<ModuleIR>::MutexCurrentKey};
  Analysis<ModuleIR>::CurrentKey = Key;
  auto &Result = AM.getResult<Analysis<ModuleIR>>(Module);
  return Result.get();
}

auto getFunctionAnalysisResult(llvm::FunctionAnalysisManager &AM,
                               llvm::AnalysisKey *Key, llvm::Function &Function)
    -> void * {
  const auto Lock = std::lock_guard{Analysis<FunctionIR>::MutexCurrentKey};
  Analysis<FunctionIR>::CurrentKey = Key;
  auto &Result = AM.getResult<Analysis<FunctionIR>>(Function);
  return Result.get();
}

auto getModuleAnalysisCachedResult(llvm::ModuleAnalysisManager &AM,
                                   llvm::AnalysisKey *Key, llvm::Module &Module)
    -> void * {
  const auto Lock = std::lock_guard{Analysis<ModuleIR>::MutexCurrentKey};
  Analysis<ModuleIR>::CurrentKey = Key;
  auto *Result = AM.getCachedResult<Analysis<ModuleIR>>(Module);
  return Result == nullptr ? nullptr : Result->get();
}

auto getFunctionAnalysisCachedResult(llvm::FunctionAnalysisManager &AM,
                                     llvm::AnalysisKey *Key,
                                     llvm::Function &Function) -> void * {
  const auto Lock = std::lock_guard{Analysis<FunctionIR>::MutexCurrentKey};
  Analysis<FunctionIR>::CurrentKey = Key;
  auto *Result = AM.getCachedResult<Analysis<FunctionIR>>(Function);
  return Result == nullptr ? nullptr : Result->get();
}

auto registerModulePass(const char *Name, size_t NameLen,
                        Pass<ModuleIR>::Entrypoint Entrypoint) -> void {
  assert(Name != nullptr && "Pass name cannot be NULL");
  auto &Passes = Pass<ModuleIR>::PassMap;
  const auto [it, inserted] =
      Passes.try_emplace(llvm::StringRef{Name, NameLen}, Entrypoint);
  assert(inserted && "Cannot register the same pass more than once");
}

auto registerFunctionPass(const char *Name, size_t NameLen,
                          Pass<FunctionIR>::Entrypoint Entrypoint) -> void {
  assert(Name != nullptr && "Pass name cannot be NULL");
  auto &Passes = Pass<FunctionIR>::PassMap;
  const auto [it, inserted] =
      Passes.try_emplace(llvm::StringRef{Name, NameLen}, Entrypoint);
  assert(inserted && "Cannot register the same pass more than once");
}

auto registerModuleAnalysis(llvm::AnalysisKey *Id,
                            Analysis<ModuleIR>::Entrypoint Entrypoint) -> void {
  auto &Analyses = Analysis<ModuleIR>::AnalysisMap;
  const auto [it, inserted] = Analyses.try_emplace(Id, Entrypoint);
  assert(inserted && "Cannot register the same analysis pass more than once");
}

auto registerFunctionAnalysis(llvm::AnalysisKey *Id,
                              Analysis<FunctionIR>::Entrypoint Entrypoint)
    -> void {
  auto &Analyses = Analysis<FunctionIR>::AnalysisMap;
  const auto [it, inserted] = Analyses.try_emplace(Id, Entrypoint);
  assert(inserted && "Cannot register the same analysis pass more than once");
}

auto llvmPluginApiVersion() -> std::uint32_t { return LLVM_PLUGIN_API_VERSION; }

auto llvmPluginRegistrar() -> void (*)(void *) {
  return reinterpret_cast<void (*)(void *)>(+[](llvm::PassBuilder &Builder) {
    // register module passes
    Builder.registerPipelineParsingCallback(
        [](llvm::StringRef PassName, llvm::ModulePassManager &PassManager,
           llvm::ArrayRef<llvm::PassBuilder::PipelineElement> /*unused*/) {
          const auto &Passes = Pass<ModuleIR>::PassMap;

          const auto Entrypoint = Passes.find(PassName);
          if (Entrypoint != Passes.end()) {
            PassManager.addPass(Pass<ModuleIR>{Entrypoint->getValue()});
            return true;
          }

          return false;
        });

    // register function passes
    Builder.registerPipelineParsingCallback(
        [](llvm::StringRef PassName, llvm::FunctionPassManager &PassManager,
           llvm::ArrayRef<llvm::PassBuilder::PipelineElement> /*unused*/) {
          const auto &Passes = Pass<FunctionIR>::PassMap;

          const auto Entrypoint = Passes.find(PassName);
          if (Entrypoint != Passes.end()) {
            PassManager.addPass(Pass<FunctionIR>{Entrypoint->getValue()});
            return true;
          }

          return false;
        });

    // register module analyses
    Builder.registerAnalysisRegistrationCallback(
        [](llvm::ModuleAnalysisManager &AM) {
          auto &Analyses = Analysis<ModuleIR>::AnalysisMap;
          const auto Lock =
              std::lock_guard{Analysis<ModuleIR>::MutexCurrentKey};

          for (const auto &Ana : Analyses) {
            Analysis<ModuleIR>::CurrentKey = Ana.first;
            AM.registerPass([&Ana] { return Analysis<ModuleIR>{Ana.second}; });
          }
        });

    // register function analyses
    Builder.registerAnalysisRegistrationCallback(
        [](llvm::FunctionAnalysisManager &AM) {
          const auto &Analyses = Analysis<FunctionIR>::AnalysisMap;
          const auto Lock =
              std::lock_guard{Analysis<FunctionIR>::MutexCurrentKey};

          for (const auto &Ana : Analyses) {
            Analysis<FunctionIR>::CurrentKey = Ana.first;
            AM.registerPass(
                [&Ana] { return Analysis<FunctionIR>{Ana.second}; });
          }
        });
  });
}
}
