#pragma once
#include "llvm/Support/raw_ostream.h"
#include <cstdint>
#include <map>
#include <string>

namespace llvm { class Module; }

namespace cobra {

struct ModuleStats {
    uint64_t functions = 0;
    uint64_t blocks = 0;
    uint64_t instructions = 0;
    uint64_t globals = 0;
};

struct PipelineStats {
    ModuleStats before;
    ModuleStats after;

    void recordBefore(const llvm::Module &M);
    void recordAfter(const llvm::Module &M);
    void print(llvm::raw_ostream &OS) const;
};

ModuleStats collectModuleStats(const llvm::Module &M);

} // namespace cobra
