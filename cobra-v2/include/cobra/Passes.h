#pragma once
#include "cobra/CobraConfig.h"
#include "cobra/RNG.h"
#include "llvm/IR/PassManager.h"

namespace cobra {

// --- Instruction Substitution ---
class InsnSubstitutionPass
    : public llvm::PassInfoMixin<InsnSubstitutionPass> {
public:
    InsnSubstitutionPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};

void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng);
void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng);

} // namespace cobra
