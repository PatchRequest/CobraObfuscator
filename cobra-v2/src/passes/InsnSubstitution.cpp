#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"

namespace cobra {

llvm::PreservedAnalyses InsnSubstitutionPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("insn-substitution"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    std::vector<llvm::Instruction *> worklist;
    for (auto &BB : F)
        for (auto &I : BB)
            worklist.push_back(&I);

    for (auto *I : worklist) {
        auto *bin = llvm::dyn_cast<llvm::BinaryOperator>(I);
        if (!bin) continue;

        llvm::IRBuilder<> B(bin);
        llvm::Value *replacement = nullptr;
        auto *lhs = bin->getOperand(0);
        auto *rhs = bin->getOperand(1);

        switch (bin->getOpcode()) {
        case llvm::Instruction::Add:
            replacement = B.CreateSub(
                lhs, B.CreateSub(
                    llvm::ConstantInt::get(rhs->getType(), 0), rhs));
            break;
        case llvm::Instruction::Sub:
            replacement = B.CreateAdd(
                lhs, B.CreateSub(
                    llvm::ConstantInt::get(rhs->getType(), 0), rhs));
            break;
        case llvm::Instruction::Xor:
            replacement = B.CreateAnd(
                B.CreateOr(lhs, rhs),
                B.CreateNot(B.CreateAnd(lhs, rhs)));
            break;
        default:
            continue;
        }

        if (replacement) {
            bin->replaceAllUsesWith(replacement);
            bin->eraseFromParent();
            changed = true;
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
