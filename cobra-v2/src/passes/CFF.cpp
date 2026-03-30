#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

#include <map>
#include <vector>

namespace cobra {

llvm::PreservedAnalyses CFFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("cff"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();
    if (F.size() < 3) return llvm::PreservedAnalyses::all();

    auto &ctx = F.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);

    // Collect all blocks
    std::vector<llvm::BasicBlock *> origBlocks;
    for (auto &BB : F)
        origBlocks.push_back(&BB);

    auto *entryBB = origBlocks[0];

    // Split entry block: keep allocas in entry, everything else goes to "cff.first"
    auto splitPoint = entryBB->begin();
    while (splitPoint != entryBB->end() &&
           llvm::isa<llvm::AllocaInst>(&*splitPoint)) {
        ++splitPoint;
    }

    llvm::BasicBlock *firstBB = entryBB->splitBasicBlock(
        splitPoint, "cff.first");

    // Create state variable alloca in entry (before the terminator that splitBasicBlock created)
    llvm::IRBuilder<> entryB(entryBB->getTerminator());
    auto *stateVar = entryB.CreateAlloca(i32Ty, nullptr, "cff.state");

    // Rebuild block list: all blocks except the entry (which is now just allocas + br)
    std::vector<llvm::BasicBlock *> flatBlocks;
    for (auto &BB : F) {
        if (&BB == entryBB) continue;
        flatBlocks.push_back(&BB);
    }

    // Assign random state IDs
    std::vector<uint32_t> stateIDs;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        stateIDs.push_back(rng.nextU32());

    // Create dispatcher block
    auto *dispatchBB = llvm::BasicBlock::Create(ctx, "cff.dispatcher", &F);
    llvm::IRBuilder<> dispB(dispatchBB);
    auto *stateVal = dispB.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *defaultBB = llvm::BasicBlock::Create(ctx, "cff.default", &F);
    llvm::IRBuilder<>(defaultBB).CreateUnreachable();
    auto *sw = dispB.CreateSwitch(stateVal, defaultBB, flatBlocks.size());

    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        sw->addCase(llvm::ConstantInt::get(i32Ty, stateIDs[i]),
                     flatBlocks[i]);
    }

    // Set initial state in entry block (replace the br that splitBasicBlock created)
    entryBB->getTerminator()->eraseFromParent();
    llvm::IRBuilder<> entryB2(entryBB);
    entryB2.CreateStore(
        llvm::ConstantInt::get(i32Ty, stateIDs[0]), stateVar);
    entryB2.CreateBr(dispatchBB);

    // Build block-to-state map
    std::map<llvm::BasicBlock *, uint32_t> blockToState;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        blockToState[flatBlocks[i]] = stateIDs[i];

    // Rewrite terminators: instead of branching to successors, set state + br to dispatcher
    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        auto *BB = flatBlocks[i];
        auto *term = BB->getTerminator();
        if (!term) continue;

        if (auto *br = llvm::dyn_cast<llvm::BranchInst>(term)) {
            if (br->isUnconditional()) {
                auto *dest = br->getSuccessor(0);
                auto it = blockToState.find(dest);
                if (it != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    B.CreateStore(
                        llvm::ConstantInt::get(i32Ty, it->second),
                        stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            } else {
                auto *cond = br->getCondition();
                auto *trueDest = br->getSuccessor(0);
                auto *falseDest = br->getSuccessor(1);
                auto trueIt = blockToState.find(trueDest);
                auto falseIt = blockToState.find(falseDest);

                if (trueIt != blockToState.end() &&
                    falseIt != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    auto *trueState = llvm::ConstantInt::get(
                        i32Ty, trueIt->second);
                    auto *falseState = llvm::ConstantInt::get(
                        i32Ty, falseIt->second);
                    auto *selected = B.CreateSelect(
                        cond, trueState, falseState);
                    B.CreateStore(selected, stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            }
        }
        // Return instructions stay as-is
    }

    // Demote PHI nodes to allocas
    // PHIs break because all predecessors now go through the dispatcher
    std::vector<llvm::PHINode *> phis;
    for (auto *BB : flatBlocks)
        for (auto &I : *BB)
            if (auto *phi = llvm::dyn_cast<llvm::PHINode>(&I))
                phis.push_back(phi);

    for (auto *phi : phis) {
        // Create alloca in entry for each phi
        llvm::IRBuilder<> allocB(entryBB, entryBB->begin());
        auto *alloca = allocB.CreateAlloca(phi->getType(), nullptr,
                                            phi->getName() + ".demoted");

        // For each incoming value, store to alloca before the branch to dispatcher
        for (unsigned j = 0; j < phi->getNumIncomingValues(); ++j) {
            auto *val = phi->getIncomingValue(j);
            auto *pred = phi->getIncomingBlock(j);
            // The predecessor's terminator should now be "store state + br dispatcher"
            // Insert the store before the terminator
            auto *predTerm = pred->getTerminator();
            llvm::IRBuilder<> B(predTerm);
            B.CreateStore(val, alloca);
        }

        // Replace phi with load from alloca (insert after all phis in the block)
        auto insertIt = phi->getParent()->getFirstNonPHIIt();
        llvm::Instruction *insertPt = &*insertIt;
        llvm::IRBuilder<> B(insertPt);
        auto *loaded = B.CreateLoad(phi->getType(), alloca);
        phi->replaceAllUsesWith(loaded);
        phi->eraseFromParent();
    }

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
