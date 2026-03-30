#pragma once

#include "llvm/IR/PassManager.h"

namespace cobra {
struct CobraConfig;
class RNG;

void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng);
void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng);

} // namespace cobra
