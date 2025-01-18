#include <memory>
#include <mutex>

#include <llvm/IR/PassManager.h>
#include <llvm/Passes/PassBuilder.h>
#include <llvm/Passes/PassPlugin.h>

#include "analysis.hh"
#include "common.hh"
#include "pass.hh"

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 14)
#include <llvm/Passes/OptimizationLevel.h>
using LlvmOptLevel = llvm::OptimizationLevel;
#else
using LlvmOptLevel = llvm::PassBuilder::OptimizationLevel;
#endif

enum class OptimizationLevel { kO0, kO1, kO2, kO3, kOs, kOz };

auto getFFIOptimizationLevel(LlvmOptLevel Opt) -> OptimizationLevel {
#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR == 10)
  if (Opt == LlvmOptLevel::O0) {
    return OptimizationLevel::kO0;
  }
  if (Opt == LlvmOptLevel::O1) {
    return OptimizationLevel::kO1;
  }
  if (Opt == LlvmOptLevel::O2) {
    return OptimizationLevel::kO2;
  }
  if (Opt == LlvmOptLevel::O3) {
    return OptimizationLevel::kO3;
  }
  if (Opt == LlvmOptLevel::Os) {
    return OptimizationLevel::kOs;
  }
#else
  // Starting from LLVM-11, llvm::OptimizationLevel::Ox is no longer
  // an enum but a global static. Using these global statics on Windows
  // would not compile, because an LLVM plugin links to opt.exe. The
  // latter doesn't export such symbols.
  if (Opt.getSpeedupLevel() == 0 && Opt.getSizeLevel() == 0) {
    return OptimizationLevel::kO0;
  }
  if (Opt.getSpeedupLevel() == 1 && Opt.getSizeLevel() == 0) {
    return OptimizationLevel::kO1;
  }
  if (Opt.getSpeedupLevel() == 2 && Opt.getSizeLevel() == 0) {
    return OptimizationLevel::kO2;
  }
  if (Opt.getSpeedupLevel() == 3 && Opt.getSizeLevel() == 0) {
    return OptimizationLevel::kO3;
  }
  if (Opt.getSpeedupLevel() == 2 && Opt.getSizeLevel() == 1) {
    return OptimizationLevel::kOs;
  }
  if (Opt.getSpeedupLevel() == 2 && Opt.getSizeLevel() == 2) {
    return OptimizationLevel::kOz;
  }
#endif
  return OptimizationLevel::kOz;
}

extern "C" {
auto moduleAnalysisManagerRegisterPass(
    llvm::ModuleAnalysisManager &AM, Analysis<ModuleIR>::DataPtr AnalysisData,
    Analysis<ModuleIR>::DataDeleter Deleter,
    Analysis<ModuleIR>::Entrypoint Entrypoint, llvm::AnalysisKey *Key) -> bool {
  const auto Lock = std::lock_guard{Analysis<ModuleIR>::MutexCurrentKey};
  Analysis<ModuleIR>::CurrentKey = Key;
  return AM.registerPass(
      [&] { return Analysis<ModuleIR>{Entrypoint, {AnalysisData, Deleter}}; });
}

auto functionAnalysisManagerRegisterPass(
    llvm::FunctionAnalysisManager &AM,
    Analysis<FunctionIR>::DataPtr AnalysisData,
    Analysis<FunctionIR>::DataDeleter Deleter,
    Analysis<FunctionIR>::Entrypoint Entrypoint, llvm::AnalysisKey *Key)
    -> bool {
  const auto Lock = std::lock_guard{Analysis<FunctionIR>::MutexCurrentKey};
  Analysis<FunctionIR>::CurrentKey = Key;
  return AM.registerPass([&] {
    return Analysis<FunctionIR>{Entrypoint, {AnalysisData, Deleter}};
  });
}

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 15)
auto passBuilderAddFullLinkTimeOptimizationLastEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerFullLinkTimeOptimizationLastEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 15)
auto passBuilderAddFullLinkTimeOptimizationEarlyEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerFullLinkTimeOptimizationEarlyEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 11)
auto passBuilderAddOptimizerLastEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerOptimizerLastEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 15)
auto passBuilderAddOptimizerEarlyEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerOptimizerEarlyEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 12)
auto passBuilderAddPipelineEarlySimplificationEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerPipelineEarlySimplificationEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 12)
auto passBuilderAddPipelineStartEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModulePassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerPipelineStartEPCallback(
      [Data = std::move(Data), Callback](llvm::ModulePassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}
#endif

auto passBuilderAddVectorizerStartEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::FunctionPassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerVectorizerStartEPCallback(
      [Data = std::move(Data), Callback](llvm::FunctionPassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}

auto passBuilderAddScalarOptimizerLateEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::FunctionPassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerScalarOptimizerLateEPCallback(
      [Data = std::move(Data), Callback](llvm::FunctionPassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}

auto passBuilderAddPeepholeEPCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::FunctionPassManager &,
                     OptimizationLevel)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerPeepholeEPCallback(
      [Data = std::move(Data), Callback](llvm::FunctionPassManager &PassManager,
                                         LlvmOptLevel Opt) {
        const auto OptFFI = getFFIOptimizationLevel(Opt);
        Callback(Data.get(), PassManager, OptFFI);
      });
}

auto passBuilderAddModuleAnalysisRegistrationCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::ModuleAnalysisManager &)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerAnalysisRegistrationCallback(
      [Data = std::move(Data), Callback](llvm::ModuleAnalysisManager &AM) {
        Callback(Data.get(), AM);
      });
}

auto passBuilderAddFunctionAnalysisRegistrationCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    void (*Callback)(const void *, llvm::FunctionAnalysisManager &)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerAnalysisRegistrationCallback(
      [Data = std::move(Data), Callback](llvm::FunctionAnalysisManager &AM) {
        Callback(Data.get(), AM);
      });
}

auto passBuilderAddModulePipelineParsingCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    bool (*Callback)(const void *, const char *, std::uintptr_t,
                     llvm::ModulePassManager &)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerPipelineParsingCallback(
      [Data = std::move(Data), Callback](
          llvm::StringRef PassName, llvm::ModulePassManager &PassManager,
          llvm::ArrayRef<llvm::PassBuilder::PipelineElement> /*unused*/) {
        return Callback(Data.get(), PassName.data(), PassName.size(),
                        PassManager);
      });
}

auto passBuilderAddFunctionPipelineParsingCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    bool (*Callback)(const void *, const char *, std::uintptr_t,
                     llvm::FunctionPassManager &)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerPipelineParsingCallback(
      [Data = std::move(Data), Callback](
          llvm::StringRef PassName, llvm::FunctionPassManager &PassManager,
          llvm::ArrayRef<llvm::PassBuilder::PipelineElement> /*unused*/) {
        return Callback(Data.get(), PassName.data(), PassName.size(),
                        PassManager);
      });
}

auto modulePassManagerAddPass(llvm::ModulePassManager &PassManager,
                              Pass<ModuleIR>::DataPtr PassData,
                              Pass<ModuleIR>::DataDeleter Deleter,
                              Pass<ModuleIR>::Entrypoint Entrypoint) -> void {
  PassManager.addPass(Pass<ModuleIR>{Entrypoint, {PassData, Deleter}});
}

auto functionPassManagerAddPass(llvm::FunctionPassManager &PassManager,
                                Pass<FunctionIR>::DataPtr PassData,
                                Pass<FunctionIR>::DataDeleter Deleter,
                                Pass<FunctionIR>::Entrypoint Entrypoint)
    -> void {
  PassManager.addPass(Pass<FunctionIR>{Entrypoint, {PassData, Deleter}});
}

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 12)
auto modulePassManagerIsEmpty(llvm::ModulePassManager &PassManager) -> bool {
  return PassManager.isEmpty();
}
#endif

#if defined(LLVM_VERSION_MAJOR) && (LLVM_VERSION_MAJOR >= 12)
auto functionPassManagerIsEmpty(llvm::FunctionPassManager &PassManager)
    -> bool {
  return PassManager.isEmpty();
}
#endif

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

auto llvmPluginApiVersion() -> std::uint32_t { return LLVM_PLUGIN_API_VERSION; }
}
