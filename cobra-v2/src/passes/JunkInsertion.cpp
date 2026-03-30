#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses JunkInsertionPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("junk-insertion"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;
    auto *i32Ty = llvm::Type::getInt32Ty(F.getContext());
    auto *i64Ty = llvm::Type::getInt64Ty(F.getContext());

    // Create junk allocas in the entry block.
    // Insert ALL allocas first (grouped together), then the initialization
    // stores, so CFF's alloca-skipping split doesn't strand an alloca in a
    // non-entry block.
    auto &entry = F.getEntryBlock();
    llvm::IRBuilder<> allocaB(&entry, entry.begin());

    int numJunkVars = rng.nextInRange(2, 5);
    std::vector<llvm::AllocaInst *> junkAllocas;
    for (int i = 0; i < numJunkVars; ++i) {
        auto *ty = rng.chance(0.5) ? i32Ty : i64Ty;
        auto *alloca = allocaB.CreateAlloca(ty, nullptr, "junk");
        junkAllocas.push_back(alloca);
        changed = true;
    }
    // Initialization stores go right after all the allocas
    for (auto *alloca : junkAllocas) {
        auto *ty = alloca->getAllocatedType();
        allocaB.CreateStore(llvm::ConstantInt::get(ty, rng.nextU32()), alloca);
    }

    // Record the first instruction after the junk alloca+store section so
    // that the scatter step never inserts a load before any of the allocas.
    llvm::Instruction *firstNonJunk = &*allocaB.GetInsertPoint();

    // Scatter junk operations through each block
    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;

        // For the entry block, start scattering only after the junk section.
        // For all blocks, skip past any leading PHI nodes (non-PHI
        // instructions cannot be inserted before PHIs in LLVM IR).
        llvm::BasicBlock::iterator startIt =
            (BB == &entry) ? llvm::BasicBlock::iterator(firstNonJunk)
                           : BB->getFirstNonPHIIt();

        // Count usable (non-terminator) instructions from startIt
        unsigned usable = 0;
        for (auto it = startIt; it != BB->end(); ++it)
            if (!it->isTerminator()) ++usable;
        if (usable == 0) continue;

        int numJunk = rng.nextInRange(1, 4);
        for (int i = 0; i < numJunk; ++i) {
            auto it = startIt;
            int steps = rng.nextInRange(0, std::max(1u, usable) - 1);
            for (int s = 0; s < steps && it != BB->end(); ++s, ++it);
            if (it == BB->end() || it->isTerminator()) continue;

            llvm::IRBuilder<> B(&*it);

            auto *alloca = junkAllocas[rng.nextU32() % junkAllocas.size()];
            auto *ty = alloca->getAllocatedType();
            auto *loaded = B.CreateLoad(ty, alloca, /*isVolatile=*/true);
            auto *junkVal = B.CreateAdd(loaded,
                llvm::ConstantInt::get(ty, rng.nextU32()));
            B.CreateStore(junkVal, alloca, /*isVolatile=*/true);
            changed = true;
        }
    }

    if (config.verbose && changed)
        llvm::errs() << "[junk-insertion] " << F.getName() << "\n";

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
