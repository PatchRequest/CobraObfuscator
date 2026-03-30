#include "cobra/Passes.h"

#include "llvm/IR/Constants.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"

namespace cobra {

llvm::Value *ConstantUnfoldPass::unfoldConstant(
    llvm::IRBuilder<> &B, llvm::ConstantInt *C) {
    auto *ty = C->getType();
    int64_t val = C->getSExtValue();

    // Decompose: val = a * b + c where a, b, c are random
    uint32_t a = rng.nextInRange(2, 50);
    int64_t b = val / static_cast<int64_t>(a);
    int64_t c = val - b * static_cast<int64_t>(a);

    auto *mulResult = B.CreateMul(
        llvm::ConstantInt::get(ty, a),
        llvm::ConstantInt::get(ty, b));
    return B.CreateAdd(mulResult, llvm::ConstantInt::get(ty, c));
}

llvm::PreservedAnalyses ConstantUnfoldPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("constant-unfold"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    std::vector<llvm::Instruction *> worklist;
    for (auto &BB : F)
        for (auto &I : BB)
            worklist.push_back(&I);

    for (auto *I : worklist) {
        // Skip phi nodes and switch — they need compile-time constants
        if (llvm::isa<llvm::PHINode>(I)) continue;
        if (llvm::isa<llvm::SwitchInst>(I)) continue;

        for (unsigned op = 0; op < I->getNumOperands(); ++op) {
            auto *ci = llvm::dyn_cast<llvm::ConstantInt>(I->getOperand(op));
            if (!ci) continue;
            // Don't unfold small constants (0, 1, -1) — too trivial
            int64_t val = ci->getSExtValue();
            if (val >= -1 && val <= 1) continue;
            // Don't unfold i1 (booleans)
            if (ci->getType()->isIntegerTy(1)) continue;

            llvm::IRBuilder<> B(I);
            auto *unfolded = unfoldConstant(B, ci);
            I->setOperand(op, unfolded);
            changed = true;
        }
    }

    if (config.verbose && changed)
        llvm::errs() << "[constant-unfold] " << F.getName() << "\n";

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
