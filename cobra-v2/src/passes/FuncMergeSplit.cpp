#include "cobra/Passes.h"

#include "llvm/IR/Constants.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"

namespace cobra {

llvm::PreservedAnalyses FuncMergeSplitPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("func-merge-split"))
        return llvm::PreservedAnalyses::all();

    auto &ctx = M.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);
    bool changed = false;

    // Collect candidate functions
    std::vector<llvm::Function *> candidates;
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        if (F.hasExternalLinkage() && F.getName() == "main") continue;
        if (F.isVarArg()) continue;
        if (F.getName().starts_with("cobra.")) continue;
        candidates.push_back(&F);
    }

    // Merge pairs with identical signatures
    std::vector<bool> merged(candidates.size(), false);
    for (size_t i = 0; i < candidates.size(); ++i) {
        if (merged[i]) continue;
        for (size_t j = i + 1; j < candidates.size(); ++j) {
            if (merged[j]) continue;
            auto *f1 = candidates[i];
            auto *f2 = candidates[j];

            if (f1->getFunctionType() != f2->getFunctionType()) continue;
            if (!rng.chance(0.5)) continue;

            // Create merged function with extra i32 selector
            auto *origTy = f1->getFunctionType();
            std::vector<llvm::Type *> paramTypes;
            paramTypes.push_back(i32Ty); // selector
            for (auto *t : origTy->params())
                paramTypes.push_back(t);

            auto *mergedTy = llvm::FunctionType::get(
                origTy->getReturnType(), paramTypes, false);
            auto *mergedFn = llvm::Function::Create(
                mergedTy, llvm::GlobalValue::InternalLinkage,
                "cobra.merged." + f1->getName() + "." + f2->getName(), M);

            auto *entryBB = llvm::BasicBlock::Create(ctx, "entry", mergedFn);
            auto *f1BB = llvm::BasicBlock::Create(ctx, "case.f1", mergedFn);
            auto *f2BB = llvm::BasicBlock::Create(ctx, "case.f2", mergedFn);
            auto *defaultBB = llvm::BasicBlock::Create(ctx, "default", mergedFn);

            // Entry: switch on selector
            llvm::IRBuilder<> entryB(entryBB);
            auto argIt = mergedFn->arg_begin();
            llvm::Value *selector = &*argIt;
            auto *sw = entryB.CreateSwitch(selector, defaultBB, 2);
            sw->addCase(llvm::ConstantInt::get(i32Ty, 0), f1BB);
            sw->addCase(llvm::ConstantInt::get(i32Ty, 1), f2BB);

            llvm::IRBuilder<>(defaultBB).CreateUnreachable();

            // Collect forwarded args (skip selector)
            std::vector<llvm::Value *> fwdArgs;
            ++argIt;
            for (; argIt != mergedFn->arg_end(); ++argIt)
                fwdArgs.push_back(&*argIt);

            // Case f1: call f1, return result
            {
                llvm::IRBuilder<> B(f1BB);
                auto *result = B.CreateCall(f1, fwdArgs);
                if (origTy->getReturnType()->isVoidTy())
                    B.CreateRetVoid();
                else
                    B.CreateRet(result);
            }
            // Case f2: call f2, return result
            {
                llvm::IRBuilder<> B(f2BB);
                auto *result = B.CreateCall(f2, fwdArgs);
                if (origTy->getReturnType()->isVoidTy())
                    B.CreateRetVoid();
                else
                    B.CreateRet(result);
            }

            // Replace call sites (skip calls inside mergedFn itself)
            auto replaceCallSites = [&](llvm::Function *origFn, uint32_t selectorVal) {
                std::vector<llvm::CallInst *> calls;
                for (auto &use : origFn->uses()) {
                    if (auto *call = llvm::dyn_cast<llvm::CallInst>(use.getUser())) {
                        if (call->getCalledFunction() == origFn &&
                            call->getParent()->getParent() != mergedFn)
                            calls.push_back(call);
                    }
                }
                for (auto *call : calls) {
                    llvm::IRBuilder<> B(call);
                    std::vector<llvm::Value *> args;
                    args.push_back(llvm::ConstantInt::get(i32Ty, selectorVal));
                    for (unsigned k = 0; k < call->arg_size(); ++k)
                        args.push_back(call->getArgOperand(k));
                    auto *newCall = B.CreateCall(mergedFn, args);
                    call->replaceAllUsesWith(newCall);
                    call->eraseFromParent();
                }
            };

            replaceCallSites(f1, 0);
            replaceCallSites(f2, 1);

            merged[i] = true;
            merged[j] = true;
            changed = true;
            break;
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
