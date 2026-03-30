#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"

namespace cobra {

llvm::PreservedAnalyses MBAPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("mba"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    std::vector<llvm::Instruction *> worklist;
    for (auto &BB : F)
        for (auto &I : BB)
            worklist.push_back(&I);

    for (auto *I : worklist) {
        auto *bin = llvm::dyn_cast<llvm::BinaryOperator>(I);
        if (!bin) continue;
        if (!bin->getType()->isIntegerTy()) continue;

        llvm::IRBuilder<> B(bin);
        llvm::Value *replacement = nullptr;
        auto *a = bin->getOperand(0);
        auto *b = bin->getOperand(1);
        auto *ty = bin->getType();

        switch (bin->getOpcode()) {
        case llvm::Instruction::Add: {
            // (a ^ b) + 2*(a & b)
            auto *xorAB = B.CreateXor(a, b);
            auto *andAB = B.CreateAnd(a, b);
            auto *shl = B.CreateShl(andAB, llvm::ConstantInt::get(ty, 1));
            replacement = B.CreateAdd(xorAB, shl);
            break;
        }
        case llvm::Instruction::Sub: {
            // (a ^ b) - 2*(~a & b)
            auto *xorAB = B.CreateXor(a, b);
            auto *notA = B.CreateNot(a);
            auto *andNotAB = B.CreateAnd(notA, b);
            auto *shl = B.CreateShl(andNotAB, llvm::ConstantInt::get(ty, 1));
            replacement = B.CreateSub(xorAB, shl);
            break;
        }
        case llvm::Instruction::Xor: {
            // (a | b) - (a & b)
            auto *orAB = B.CreateOr(a, b);
            auto *andAB = B.CreateAnd(a, b);
            replacement = B.CreateSub(orAB, andAB);
            break;
        }
        case llvm::Instruction::Or: {
            // (a ^ b) + (a & b)
            auto *xorAB = B.CreateXor(a, b);
            auto *andAB = B.CreateAnd(a, b);
            replacement = B.CreateAdd(xorAB, andAB);
            break;
        }
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
