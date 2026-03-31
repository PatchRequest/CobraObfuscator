#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/GlobalVariable.h"

#include <sstream>
#include <iomanip>

namespace cobra {

static std::string randomName(RNG &rng, const std::string &prefix) {
    std::stringstream ss;
    ss << prefix << std::hex << rng.nextU32() << rng.nextU32();
    return ss.str();
}

llvm::PreservedAnalyses SymbolStripPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("symbol-strip"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    unsigned renamed = 0;

    // Rename internal/private functions
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        if (F.getName() == "main") continue;
        if (F.getName().starts_with("cobra.")) continue;
        if (F.hasExternalLinkage()) continue;

        F.setName(randomName(rng, "f_"));
        renamed++;
        changed = true;
    }

    // Rename internal global variables
    for (auto &GV : M.globals()) {
        if (GV.hasExternalLinkage()) continue;
        if (GV.getName().starts_with("cobra.")) continue;
        if (GV.getName().empty()) continue;

        GV.setName(randomName(rng, "g_"));
        changed = true;
    }

    // Strip names from basic blocks and local values
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        for (auto &BB : F) {
            BB.setName("");
            for (auto &I : BB) {
                if (!I.getType()->isVoidTy() && I.hasName())
                    I.setName("");
            }
        }
    }

    if (config.verbose && changed)
        llvm::errs() << "[symbol-strip] renamed " << renamed << " functions\n";

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
