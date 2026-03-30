#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/GlobalVariable.h"

namespace cobra {

llvm::PreservedAnalyses StringEncryptPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("string-encrypt"))
        return llvm::PreservedAnalyses::all();

    auto &ctx = M.getContext();
    auto *i8Ty = llvm::Type::getInt8Ty(ctx);
    auto *i64Ty = llvm::Type::getInt64Ty(ctx);
    auto *ptrTy = llvm::PointerType::getUnqual(ctx);
    bool changed = false;

    std::vector<llvm::GlobalVariable *> stringGlobals;
    for (auto &GV : M.globals()) {
        if (!GV.hasInitializer()) continue;
        if (!GV.isConstant()) continue;
        auto *init = llvm::dyn_cast<llvm::ConstantDataArray>(GV.getInitializer());
        if (!init) continue;
        if (!init->isString() && !init->isCString()) continue;
        stringGlobals.push_back(&GV);
    }

    for (auto *GV : stringGlobals) {
        auto *init = llvm::cast<llvm::ConstantDataArray>(GV->getInitializer());
        auto strData = init->getAsString();
        size_t len = strData.size();

        // Generate XOR key
        std::vector<uint8_t> key(len);
        for (size_t i = 0; i < len; ++i)
            key[i] = static_cast<uint8_t>(rng.nextU32());

        // Encrypt
        std::vector<uint8_t> encrypted(len);
        for (size_t i = 0; i < len; ++i)
            encrypted[i] = static_cast<uint8_t>(strData[i]) ^ key[i];

        // Create encrypted global (mutable)
        auto *encType = llvm::ArrayType::get(i8Ty, len);
        auto *encInit = llvm::ConstantDataArray::get(ctx, encrypted);
        auto *encGV = new llvm::GlobalVariable(
            M, encType, false, llvm::GlobalValue::PrivateLinkage,
            encInit, "cobra.enc." + GV->getName());

        // Create key global (constant)
        auto *keyInit = llvm::ConstantDataArray::get(ctx, key);
        auto *keyGV = new llvm::GlobalVariable(
            M, encType, true, llvm::GlobalValue::PrivateLinkage,
            keyInit, "cobra.key." + GV->getName());

        // Create a guard flag so decryption only happens once
        auto *i1Ty = llvm::Type::getInt1Ty(ctx);
        auto *guardGV = new llvm::GlobalVariable(
            M, i1Ty, false, llvm::GlobalValue::PrivateLinkage,
            llvm::ConstantInt::getFalse(ctx),
            "cobra.guard." + GV->getName());

        // Create decryption function
        // Mark optnone+noinline so LLVM's backend optimizer doesn't try
        // to transform our simple XOR loop (loop-simplify crashes on it)
        auto *decFnTy = llvm::FunctionType::get(ptrTy, {}, false);
        auto *decFn = llvm::Function::Create(
            decFnTy, llvm::GlobalValue::PrivateLinkage,
            "cobra.dec." + GV->getName(), M);
        decFn->addFnAttr(llvm::Attribute::OptimizeNone);
        decFn->addFnAttr(llvm::Attribute::NoInline);

        auto *entryBB = llvm::BasicBlock::Create(ctx, "entry", decFn);
        auto *loopBB = llvm::BasicBlock::Create(ctx, "loop", decFn);
        auto *exitBB = llvm::BasicBlock::Create(ctx, "exit", decFn);

        // Entry: check guard flag, skip decryption if already done
        llvm::IRBuilder<> entryB(entryBB);
        auto *alreadyDecrypted = entryB.CreateLoad(i1Ty, guardGV);
        entryB.CreateCondBr(alreadyDecrypted, exitBB, loopBB);

        llvm::IRBuilder<> loopB(loopBB);
        auto *idx = loopB.CreatePHI(i64Ty, 2, "idx");
        idx->addIncoming(llvm::ConstantInt::get(i64Ty, 0), entryBB);

        auto *encPtr = loopB.CreateGEP(i8Ty, encGV, idx);
        auto *keyPtr = loopB.CreateGEP(i8Ty, keyGV, idx);
        auto *encByte = loopB.CreateLoad(i8Ty, encPtr);
        auto *keyByte = loopB.CreateLoad(i8Ty, keyPtr);
        auto *decByte = loopB.CreateXor(encByte, keyByte);
        loopB.CreateStore(decByte, encPtr);

        auto *nextIdx = loopB.CreateAdd(idx, llvm::ConstantInt::get(i64Ty, 1));
        idx->addIncoming(nextIdx, loopBB);
        auto *done = loopB.CreateICmpEQ(nextIdx, llvm::ConstantInt::get(i64Ty, len));
        loopB.CreateCondBr(done, exitBB, loopBB);

        // Exit: set guard flag and return pointer
        llvm::IRBuilder<> exitB(exitBB);
        exitB.CreateStore(llvm::ConstantInt::getTrue(ctx), guardGV);
        exitB.CreateRet(encGV);

        // Replace uses of original global
        std::vector<llvm::Use *> uses;
        for (auto &use : GV->uses())
            uses.push_back(&use);

        for (auto *use : uses) {
            auto *user = use->getUser();
            if (auto *inst = llvm::dyn_cast<llvm::Instruction>(user)) {
                llvm::IRBuilder<> useB(inst);
                auto *decrypted = useB.CreateCall(decFn);
                use->set(decrypted);
            } else if (auto *ce = llvm::dyn_cast<llvm::ConstantExpr>(user)) {
                std::vector<llvm::Use *> ceUses;
                for (auto &ceUse : ce->uses())
                    ceUses.push_back(&ceUse);
                for (auto *ceUse : ceUses) {
                    if (auto *inst = llvm::dyn_cast<llvm::Instruction>(ceUse->getUser())) {
                        llvm::IRBuilder<> useB(inst);
                        auto *decrypted = useB.CreateCall(decFn);
                        ceUse->set(decrypted);
                    }
                }
            }
        }

        // Erase the original global if it has no remaining users
        if (GV->use_empty())
            GV->eraseFromParent();

        changed = true;
    }

    if (config.verbose && changed)
        llvm::errs() << "[string-encrypt] encrypted " << stringGlobals.size() << " strings\n";

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
