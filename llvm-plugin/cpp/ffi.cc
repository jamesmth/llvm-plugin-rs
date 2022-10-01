#include <memory>
#include <mutex>

#include <llvm/IR/PassManager.h>
#include <llvm/Passes/PassBuilder.h>
#include <llvm/Passes/PassPlugin.h>

#include "analysis.hh"
#include "common.hh"
#include "pass.hh"

extern "C" {
auto moduleAnalysisManagerRegisterPass(
    llvm::ModuleAnalysisManager &AM, Analysis<ModuleIR>::DataPtr AnalysisData,
    Analysis<ModuleIR>::DataDeleter Deleter,
    Analysis<ModuleIR>::Entrypoint Entrypoint, llvm::AnalysisKey *Key) -> bool {
  const auto Lock = std::lock_guard{Analysis<ModuleIR>::MutexCurrentKey};
  Analysis<ModuleIR>::CurrentKey = Key;
  return AM.registerPass([&] {
    return Analysis<ModuleIR>{Entrypoint, {AnalysisData, Deleter}};
  });
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

auto passBuilderAddModuleAnalysisRegistrationCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    bool (*Callback)(const void *, llvm::ModuleAnalysisManager &)) -> void {
  const auto Data = std::shared_ptr<const void>(DataPtr, Deleter);

  Builder.registerAnalysisRegistrationCallback(
      [Data = std::move(Data), Callback](llvm::ModuleAnalysisManager &AM) {
        Callback(Data.get(), AM);
      });
}

auto passBuilderAddFunctionAnalysisRegistrationCallback(
    llvm::PassBuilder &Builder, const void *DataPtr,
    void (*Deleter)(const void *),
    bool (*Callback)(const void *, llvm::FunctionAnalysisManager &)) -> void {
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

#if defined(LLVM10_0) || defined(LLVM11_0)
#else
auto modulePassManagerIsEmpty(llvm::ModulePassManager &PassManager) -> bool {
  return PassManager.isEmpty();
}
#endif

#if defined(LLVM10_0) || defined(LLVM11_0)
#else
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
