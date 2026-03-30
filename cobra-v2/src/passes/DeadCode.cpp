#include "cobra/Passes.h"
#include "cobra/OpaquePredicates.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses DeadCodePass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("dead-code"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;

    // Collect blocks to process (skip entry — splitting it is tricky)
    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        if (&BB != &F.getEntryBlock())
            blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;
        if (!rng.chance(0.5)) continue;  // 50% of blocks

        // Create a fake block with junk instructions
        auto *fakeBB = llvm::BasicBlock::Create(
            F.getContext(), "dead." + BB->getName(), &F);
        llvm::IRBuilder<> fakeB(fakeBB);

        auto *i32Ty = fakeB.getInt32Ty();
        auto *junk1 = fakeB.CreateAdd(
            llvm::ConstantInt::get(i32Ty, rng.nextU32()),
            llvm::ConstantInt::get(i32Ty, rng.nextU32()));
        auto *junk2 = fakeB.CreateMul(junk1,
            llvm::ConstantInt::get(i32Ty, rng.nextU32()));
        (void)junk2;
        fakeB.CreateBr(BB); // fake block jumps to real block

        // Split BB before its first instruction; the new block gets all
        // predecessors of BB and an unconditional branch to BB.
        auto *splitBB = BB->splitBasicBlockBefore(
            BB->begin(), "opaque." + BB->getName());

        // Replace the unconditional branch inserted by splitBasicBlockBefore
        // with an opaque-predicate conditional branch.
        splitBB->getTerminator()->eraseFromParent();
        llvm::IRBuilder<> B(splitBB);
        auto *cond = createOpaqueTrue(B, rng);
        B.CreateCondBr(cond, BB, fakeBB);

        changed = true;
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
