#pragma once
#include "cobra/CobraConfig.h"
#include "cobra/RNG.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/PassManager.h"

namespace cobra {

// --- Constant Unfolding ---
class ConstantUnfoldPass : public llvm::PassInfoMixin<ConstantUnfoldPass> {
public:
    ConstantUnfoldPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
    llvm::Value *unfoldConstant(llvm::IRBuilder<> &B, llvm::ConstantInt *C);
};

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

// --- MBA (Mixed Boolean-Arithmetic) ---
class MBAPass : public llvm::PassInfoMixin<MBAPass> {
public:
    MBAPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};

// --- Dead Code (Opaque Predicates) ---
class DeadCodePass : public llvm::PassInfoMixin<DeadCodePass> {
public:
    DeadCodePass(CobraConfig &config, RNG &rng)
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
