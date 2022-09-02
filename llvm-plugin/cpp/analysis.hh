#pragma once

#include <memory>
#include <mutex>
#include <type_traits>
#include <unordered_map>
#include <utility>

#include <llvm/IR/PassManager.h>

#include "common.hh"

namespace {
using ResultPtr = void *;
using ResultDeleter = void (*)(ResultPtr);

template <typename IR>
struct Analysis : public llvm::AnalysisInfoMixin<Analysis<IR>> {
  using Result =
      std::unique_ptr<std::remove_pointer_t<ResultPtr>, ResultDeleter>;
  using Entrypoint = void (*)(typename IR::Unit &,
                              typename IR::AnalysisManager &, ResultPtr &,
                              ResultDeleter &);

  Analysis(Entrypoint Func) { this->Func = Func; }

  auto run(typename IR::Unit &IrUnit, typename IR::AnalysisManager &AM)
      -> Result {
    auto *Ptr = (ResultPtr) nullptr;
    auto Deleter = (ResultDeleter) nullptr;
    this->Func(IrUnit, AM, Ptr, Deleter);

    return {Ptr, Deleter};
  }

  static auto ID() -> llvm::AnalysisKey * { return CurrentKey; }

  static inline auto AnalysisMap =
      std::unordered_map<llvm::AnalysisKey *, Entrypoint>{};

  static inline auto CurrentKey = (llvm::AnalysisKey *)nullptr;
  static inline auto MutexCurrentKey = std::mutex{};

private:
  Entrypoint Func;

  friend struct llvm::AnalysisInfoMixin<Analysis<IR>>;
};

} // namespace
