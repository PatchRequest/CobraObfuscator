#include "cobra/Passes.h"
#include "cobra/OpaquePredicates.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses BogusCFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("bogus-cf"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;

    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;
        if (!rng.chance(0.4)) continue;  // 40% chance per block

        // Create bogus block with fake computation
        auto *bogusBB = llvm::BasicBlock::Create(
            F.getContext(), "bogus." + BB->getName(), &F);
        llvm::IRBuilder<> bogusB(bogusBB);

        auto *i32Ty = bogusB.getInt32Ty();
        auto *c1 = llvm::ConstantInt::get(i32Ty, rng.nextU32());
        auto *c2 = llvm::ConstantInt::get(i32Ty, rng.nextU32());
        auto *fakeOp = bogusB.CreateXor(c1, c2);
        auto *fakeOp2 = bogusB.CreateAdd(fakeOp, c1);
        (void)fakeOp2;
        bogusB.CreateBr(BB);  // bogus path falls through to real block

        // Split before BB and insert opaque predicate
        auto *guardBB = BB->splitBasicBlockBefore(
            BB->begin(), "guard." + BB->getName());
        guardBB->getTerminator()->eraseFromParent();

        llvm::IRBuilder<> B(guardBB);
        auto *cond = createOpaqueTrue(B, rng);
        B.CreateCondBr(cond, BB, bogusBB);

        changed = true;
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
