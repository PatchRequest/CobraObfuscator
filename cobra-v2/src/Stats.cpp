#include "cobra/Stats.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Function.h"

namespace cobra {

ModuleStats collectModuleStats(const llvm::Module &M) {
    ModuleStats s;
    for (auto &GV : M.globals()) { (void)GV; s.globals++; }
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        s.functions++;
        for (auto &BB : F) {
            s.blocks++;
            for (auto &I : BB) { (void)I; s.instructions++; }
        }
    }
    return s;
}

void PipelineStats::recordBefore(const llvm::Module &M) { before = collectModuleStats(M); }
void PipelineStats::recordAfter(const llvm::Module &M) { after = collectModuleStats(M); }

void PipelineStats::print(llvm::raw_ostream &OS) const {
    OS << "\n=== CobraObfuscator Statistics ===\n\n";
    OS << "  Module before:\n";
    OS << "    Functions:    " << before.functions << "\n";
    OS << "    Basic blocks: " << before.blocks << "\n";
    OS << "    Instructions: " << before.instructions << "\n";
    OS << "    Globals:      " << before.globals << "\n\n";
    OS << "  Module after:\n";
    OS << "    Functions:    " << after.functions;
    if (after.functions != before.functions) OS << " (+" << (int64_t)(after.functions - before.functions) << ")";
    OS << "\n    Basic blocks: " << after.blocks;
    if (after.blocks != before.blocks) OS << " (+" << (int64_t)(after.blocks - before.blocks) << ")";
    OS << "\n    Instructions: " << after.instructions;
    if (after.instructions != before.instructions) OS << " (+" << (int64_t)(after.instructions - before.instructions) << ")";
    OS << "\n    Globals:      " << after.globals;
    if (after.globals != before.globals) OS << " (+" << (int64_t)(after.globals - before.globals) << ")";
    OS << "\n\n";
}

} // namespace cobra
