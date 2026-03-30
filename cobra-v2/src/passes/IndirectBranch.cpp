#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Constants.h"

#include <map>
#include <vector>

namespace cobra {

llvm::PreservedAnalyses IndirectBranchPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("indirect-branch"))
        return llvm::PreservedAnalyses::all();

    auto &ctx = M.getContext();
    auto *ptrTy = llvm::PointerType::getUnqual(ctx);
    bool changed = false;

    // Collect all internal functions that can be indirected
    std::vector<llvm::Function *> targets;
    std::map<llvm::Function *, size_t> funcToIdx;

    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        if (F.getName() == "main") continue;
        if (F.getName().starts_with("cobra.")) continue;
        funcToIdx[&F] = targets.size();
        targets.push_back(&F);
    }

    if (targets.empty())
        return llvm::PreservedAnalyses::all();

    // Create global function pointer table: [N x ptr]
    auto *tableTy = llvm::ArrayType::get(ptrTy, targets.size());
    std::vector<llvm::Constant *> tableEntries;
    for (auto *F : targets)
        tableEntries.push_back(F);

    auto *tableInit = llvm::ConstantArray::get(tableTy, tableEntries);
    auto *table = new llvm::GlobalVariable(
        M, tableTy, /*isConstant=*/true, llvm::GlobalValue::PrivateLinkage,
        tableInit, "cobra.fptable");

    auto *i64Ty = llvm::Type::getInt64Ty(ctx);
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);

    // Replace direct calls with indirect calls through table
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        for (auto &BB : F) {
            std::vector<llvm::CallInst *> calls;
            for (auto &I : BB) {
                auto *call = llvm::dyn_cast<llvm::CallInst>(&I);
                if (!call) continue;
                auto *callee = call->getCalledFunction();
                if (!callee) continue;
                auto it = funcToIdx.find(callee);
                if (it == funcToIdx.end()) continue;
                if (!rng.chance(0.7)) continue;
                calls.push_back(call);
            }

            for (auto *call : calls) {
                auto *callee = call->getCalledFunction();
                size_t idx = funcToIdx[callee];

                llvm::IRBuilder<> B(call);
                // GEP into [N x ptr] array: two indices — [0][idx]
                auto *zero = llvm::ConstantInt::get(i32Ty, 0);
                auto *idxVal = llvm::ConstantInt::get(i64Ty, idx);
                auto *gep = B.CreateGEP(tableTy, table,
                    {zero, idxVal}, "fp.slot");
                auto *fp = B.CreateLoad(ptrTy, gep, "fp");

                std::vector<llvm::Value *> args;
                for (unsigned i = 0; i < call->arg_size(); ++i)
                    args.push_back(call->getArgOperand(i));

                auto *newCall = B.CreateCall(
                    callee->getFunctionType(), fp, args);
                newCall->setCallingConv(call->getCallingConv());
                call->replaceAllUsesWith(newCall);
                call->eraseFromParent();
                changed = true;
            }
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
