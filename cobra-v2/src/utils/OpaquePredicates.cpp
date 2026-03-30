#include "cobra/OpaquePredicates.h"
#include "cobra/RNG.h"

#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::Value *createOpaqueTrue(llvm::IRBuilderBase &B, RNG &rng) {
    auto *func = B.GetInsertBlock()->getParent();
    auto *i32Ty = B.getInt32Ty();
    llvm::Value *x = nullptr;

    // Try to find an i32 argument
    for (auto &arg : func->args()) {
        if (arg.getType()->isIntegerTy(32)) {
            x = &arg;
            break;
        }
    }

    // If no i32 arg, use a stack alloca + volatile load
    if (!x) {
        auto &entry = func->getEntryBlock();
        llvm::IRBuilder<> allocaB(&entry, entry.begin());
        auto *alloca = allocaB.CreateAlloca(i32Ty);
        allocaB.CreateStore(llvm::ConstantInt::get(i32Ty, 0), alloca);
        x = B.CreateLoad(i32Ty, alloca, /*isVolatile=*/true);
    }

    uint32_t variant = rng.nextU32() % 3;
    switch (variant) {
    case 0: {
        // x*(x-1) % 2 == 0
        auto *xm1 = B.CreateSub(x, llvm::ConstantInt::get(i32Ty, 1));
        auto *mul = B.CreateMul(x, xm1);
        auto *rem = B.CreateURem(mul, llvm::ConstantInt::get(i32Ty, 2));
        return B.CreateICmpEQ(rem, llvm::ConstantInt::get(i32Ty, 0));
    }
    case 1: {
        // (x | 1) != 0 — always true since bit 0 is set
        auto *ored = B.CreateOr(x, llvm::ConstantInt::get(i32Ty, 1));
        return B.CreateICmpNE(ored, llvm::ConstantInt::get(i32Ty, 0));
    }
    default: {
        // (x^2 + x) % 2 == 0 — x^2+x = x(x+1), always even
        auto *xp1 = B.CreateAdd(x, llvm::ConstantInt::get(i32Ty, 1));
        auto *mul = B.CreateMul(x, xp1);
        auto *rem = B.CreateURem(mul, llvm::ConstantInt::get(i32Ty, 2));
        return B.CreateICmpEQ(rem, llvm::ConstantInt::get(i32Ty, 0));
    }
    }
}

llvm::Value *createOpaqueFalse(llvm::IRBuilderBase &B, RNG &rng) {
    return B.CreateNot(createOpaqueTrue(B, rng));
}

} // namespace cobra
