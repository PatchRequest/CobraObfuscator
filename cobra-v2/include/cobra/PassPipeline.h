#pragma once
#include "cobra/CobraConfig.h"

namespace llvm {
class Module;
} // namespace llvm

namespace cobra {

void runPipeline(llvm::Module &M, CobraConfig &config);

} // namespace cobra
