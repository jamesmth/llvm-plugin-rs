#pragma once

#include <llvm/ADT/StringMap.h>
#include <llvm/IR/PassManager.h>

#include "common.hh"

namespace {
enum class PreservedAnalyses {
  kAll,
  kNone,
};

template <typename IR> struct Pass : public llvm::PassInfoMixin<Pass<IR>> {
  using Entrypoint = PreservedAnalyses (*)(typename IR::Unit &,
                                           typename IR::AnalysisManager &);

  Pass(Entrypoint Func) { this->Func = Func; }

  auto run(typename IR::Unit &IrUnit, typename IR::AnalysisManager &AM)
      -> llvm::PreservedAnalyses {
    return (this->Func(IrUnit, AM) == PreservedAnalyses::kAll
                ? llvm::PreservedAnalyses::all()
                : llvm::PreservedAnalyses::none());
  }

  static inline auto PassMap = llvm::StringMap<Entrypoint>{};

private:
  Entrypoint Func;
};

} // namespace
