#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses AntiTamperPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("anti-tamper"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();
    if (F.getName() == "main") return llvm::PreservedAnalyses::all();
    if (F.getName().starts_with("cobra.")) return llvm::PreservedAnalyses::all();
    if (F.size() < 2) return llvm::PreservedAnalyses::all();

    auto &ctx = F.getContext();
    auto *M = F.getParent();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);
    auto *voidTy = llvm::Type::getVoidTy(ctx);

    // Compute hash from IR structure
    uint32_t blockCount = 0, insnCount = 0;
    for (auto &BB : F) {
        blockCount++;
        for (auto &I : BB) { (void)I; insnCount++; }
    }
    uint32_t salt = rng.nextU32();
    uint32_t expectedHash = blockCount ^ insnCount ^ salt;

    // Create globals for hash, salt, and encoded counts
    auto *hashGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, expectedHash),
        "cobra.tamper.hash." + F.getName().str());
    auto *saltGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, salt),
        "cobra.tamper.salt." + F.getName().str());
    auto *countGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, blockCount ^ insnCount),
        "cobra.tamper.counts." + F.getName().str());

    // Get or declare abort()
    auto abortCallee = M->getOrInsertFunction("abort",
        llvm::FunctionType::get(voidTy, false));
    auto *abortFn = llvm::cast<llvm::Function>(abortCallee.getCallee());

    // Split entry block: keep allocas in entry, real code goes to "cobra.tamper.ok"
    auto &entryBB = F.getEntryBlock();
    auto splitIt = entryBB.begin();
    while (splitIt != entryBB.end() && llvm::isa<llvm::AllocaInst>(&*splitIt))
        ++splitIt;

    auto *realBB = entryBB.splitBasicBlock(splitIt, "cobra.tamper.ok");
    auto *trapBB = llvm::BasicBlock::Create(ctx, "cobra.tamper.fail", &F);

    // Replace entry's unconditional br with the integrity check
    entryBB.getTerminator()->eraseFromParent();
    llvm::IRBuilder<> B(&entryBB);

    auto *loadedHash   = B.CreateLoad(i32Ty, hashGV);
    auto *loadedSalt   = B.CreateLoad(i32Ty, saltGV);
    auto *loadedCounts = B.CreateLoad(i32Ty, countGV);
    auto *computed     = B.CreateXor(loadedCounts, loadedSalt);
    auto *match        = B.CreateICmpEQ(computed, loadedHash);
    B.CreateCondBr(match, realBB, trapBB);

    // Trap block: call abort then unreachable
    llvm::IRBuilder<> trapB(trapBB);
    trapB.CreateCall(abortFn);
    trapB.CreateUnreachable();

    if (config.verbose)
        llvm::errs() << "[anti-tamper] " << F.getName() << "\n";

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
