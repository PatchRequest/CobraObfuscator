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

    // Create junk allocas in the entry block
    auto &entry = F.getEntryBlock();
    llvm::IRBuilder<> allocaB(&entry, entry.begin());

    int numJunkVars = rng.nextInRange(2, 5);
    std::vector<llvm::AllocaInst *> junkAllocas;
    for (int i = 0; i < numJunkVars; ++i) {
        auto *ty = rng.chance(0.5) ? i32Ty : i64Ty;
        auto *alloca = allocaB.CreateAlloca(ty, nullptr, "junk");
        allocaB.CreateStore(
            llvm::ConstantInt::get(ty, rng.nextU32()), alloca);
        junkAllocas.push_back(alloca);
        changed = true;
    }

    // Scatter junk operations through each block
    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;

        int numJunk = rng.nextInRange(1, 4);
        for (int i = 0; i < numJunk; ++i) {
            auto it = BB->begin();
            int steps = rng.nextInRange(0, std::max(1u,
                (uint32_t)BB->size() - 1));
            for (int s = 0; s < steps && it != BB->end(); ++s, ++it);
            if (it->isTerminator()) continue;

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

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
