#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

#include <map>
#include <vector>

namespace cobra {

// Strategy 0: Switch dispatcher (original)
static void buildSwitchDispatcher(
    llvm::BasicBlock *dispatchBB, llvm::BasicBlock *defaultBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    const std::vector<uint32_t> &stateIDs,
    llvm::Type *i32Ty) {
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *sw = B.CreateSwitch(stateVal, defaultBB, flatBlocks.size());
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        sw->addCase(llvm::cast<llvm::ConstantInt>(
            llvm::ConstantInt::get(i32Ty, stateIDs[i])), flatBlocks[i]);
}

// Strategy 1: If-else chain dispatcher
static void buildIfElseDispatcher(
    llvm::BasicBlock *dispatchBB, llvm::BasicBlock *defaultBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    const std::vector<uint32_t> &stateIDs,
    llvm::Type *i32Ty, llvm::Function &F) {
    auto &ctx = F.getContext();
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");

    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        auto *cmp = B.CreateICmpEQ(stateVal,
            llvm::ConstantInt::get(i32Ty, stateIDs[i]));
        if (i == flatBlocks.size() - 1) {
            B.CreateCondBr(cmp, flatBlocks[i], defaultBB);
        } else {
            auto *nextCheckBB = llvm::BasicBlock::Create(
                ctx, "cff.check." + std::to_string(i), &F);
            B.CreateCondBr(cmp, flatBlocks[i], nextCheckBB);
            B.SetInsertPoint(nextCheckBB);
        }
    }
}

// Strategy 2: XOR + lookup table dispatcher
static void buildLookupDispatcher(
    llvm::BasicBlock *dispatchBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    uint32_t xorKey,
    llvm::Type *i32Ty, llvm::Function &F) {
    auto &ctx = F.getContext();

    // Build blockaddress array
    std::vector<llvm::Constant *> blockAddrs;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        blockAddrs.push_back(llvm::BlockAddress::get(&F, flatBlocks[i]));

    auto *ptrTy = llvm::PointerType::getUnqual(ctx);
    auto *tableTy = llvm::ArrayType::get(ptrTy, flatBlocks.size());
    auto *tableInit = llvm::ConstantArray::get(tableTy, blockAddrs);
    auto *tableGV = new llvm::GlobalVariable(
        *F.getParent(), tableTy, true, llvm::GlobalValue::PrivateLinkage,
        tableInit, "cff.table." + F.getName().str());

    // Dispatcher: load state, XOR with key, mod tableSize, load from table, indirectbr
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *xored = B.CreateXor(stateVal, llvm::ConstantInt::get(i32Ty, xorKey));
    auto *idx = B.CreateURem(xored,
        llvm::ConstantInt::get(i32Ty, flatBlocks.size()));
    auto *idx64 = B.CreateZExt(idx, llvm::Type::getInt64Ty(ctx));
    auto *gep = B.CreateGEP(ptrTy, tableGV, idx64);
    auto *target = B.CreateLoad(ptrTy, gep);
    auto *ibr = B.CreateIndirectBr(target, flatBlocks.size());
    for (auto *BB : flatBlocks)
        ibr->addDestination(BB);
}

llvm::PreservedAnalyses CFFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("cff"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();
    if (F.size() < 3) return llvm::PreservedAnalyses::all();

    auto &ctx = F.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);

    // Choose dispatcher strategy: 0=switch, 1=if-else, 2=lookup
    int strategy = rng.nextU32() % 3;

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

    entryBB->splitBasicBlock(splitPoint, "cff.first");

    // Create state variable alloca in entry (before the terminator that splitBasicBlock created)
    llvm::IRBuilder<> entryB(entryBB->getTerminator());
    auto *stateVar = entryB.CreateAlloca(i32Ty, nullptr, "cff.state");

    // Rebuild block list: all blocks except the entry (which is now just allocas + br)
    std::vector<llvm::BasicBlock *> flatBlocks;
    for (auto &BB : F) {
        if (&BB == entryBB) continue;
        flatBlocks.push_back(&BB);
    }

    // Assign state IDs based on strategy
    std::vector<uint32_t> stateIDs;
    uint32_t lookupXorKey = rng.nextU32();

    if (strategy == 2) {
        // Lookup: stateIDs[i] = i ^ xorKey, so (stateID ^ xorKey) % N == i
        for (size_t i = 0; i < flatBlocks.size(); ++i)
            stateIDs.push_back(static_cast<uint32_t>(i) ^ lookupXorKey);
    } else {
        // Switch/if-else: random state IDs
        for (size_t i = 0; i < flatBlocks.size(); ++i)
            stateIDs.push_back(rng.nextU32());
    }

    // Create dispatcher and default blocks
    auto *dispatchBB = llvm::BasicBlock::Create(ctx, "cff.dispatcher", &F);
    auto *defaultBB = llvm::BasicBlock::Create(ctx, "cff.default", &F);
    llvm::IRBuilder<>(defaultBB).CreateUnreachable();

    // Build dispatcher based on chosen strategy
    switch (strategy) {
    case 0:
        buildSwitchDispatcher(dispatchBB, defaultBB, stateVar,
                              flatBlocks, stateIDs, i32Ty);
        break;
    case 1:
        buildIfElseDispatcher(dispatchBB, defaultBB, stateVar,
                              flatBlocks, stateIDs, i32Ty, F);
        break;
    case 2:
        buildLookupDispatcher(dispatchBB, stateVar,
                              flatBlocks, lookupXorKey, i32Ty, F);
        break;
    }

    // Set initial state in entry block (replace the br that splitBasicBlock created)
    entryBB->getTerminator()->eraseFromParent();
    llvm::IRBuilder<> entryB2(entryBB);
    entryB2.CreateStore(llvm::ConstantInt::get(i32Ty, stateIDs[0]), stateVar);
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
                auto it = blockToState.find(br->getSuccessor(0));
                if (it != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    B.CreateStore(llvm::ConstantInt::get(i32Ty, it->second), stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            } else {
                auto *cond = br->getCondition();
                auto trueIt = blockToState.find(br->getSuccessor(0));
                auto falseIt = blockToState.find(br->getSuccessor(1));
                if (trueIt != blockToState.end() && falseIt != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    auto *selected = B.CreateSelect(cond,
                        llvm::ConstantInt::get(i32Ty, trueIt->second),
                        llvm::ConstantInt::get(i32Ty, falseIt->second));
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
            auto *predTerm = phi->getIncomingBlock(j)->getTerminator();
            llvm::IRBuilder<> B(predTerm);
            B.CreateStore(phi->getIncomingValue(j), alloca);
        }

        // Replace phi with load from alloca (insert after all phis in the block)
        auto insertIt = phi->getParent()->getFirstNonPHIIt();
        llvm::IRBuilder<> B(&*insertIt);
        auto *loaded = B.CreateLoad(phi->getType(), alloca);
        phi->replaceAllUsesWith(loaded);
        phi->eraseFromParent();
    }

    if (config.verbose) {
        const char *names[] = {"switch", "if-else", "lookup"};
        llvm::errs() << "[cff:" << names[strategy] << "] " << F.getName() << "\n";
    }

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
