#include "cobra/PassPipeline.h"
#include "cobra/Passes.h"
#include "cobra/RNG.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Support/raw_ostream.h"

namespace cobra {

void runPipeline(llvm::Module &M, CobraConfig &config) {
    for (int iter = 0; iter < config.iterations; ++iter) {
        if (config.verbose)
            llvm::errs() << "=== Iteration " << (iter + 1)
                         << "/" << config.iterations << " ===\n";

        RNG rng(config.seed + iter);

        llvm::LoopAnalysisManager LAM;
        llvm::FunctionAnalysisManager FAM;
        llvm::CGSCCAnalysisManager CGAM;
        llvm::ModuleAnalysisManager MAM;

        llvm::PassBuilder PB;
        PB.registerModuleAnalyses(MAM);
        PB.registerCGSCCAnalyses(CGAM);
        PB.registerFunctionAnalyses(FAM);
        PB.registerLoopAnalyses(LAM);
        PB.crossRegisterProxies(LAM, FAM, CGAM, MAM);

        // Phase 1: Pre-CFF module passes (string-encrypt)
        {
            llvm::ModulePassManager MPM;
            // string-encrypt registered here in later task
            MPM.run(M, MAM);
        }

        // Phase 2: Function passes
        {
            llvm::ModulePassManager MPM;
            llvm::FunctionPassManager FPM;
            registerFunctionPasses(FPM, config, rng);
            MPM.addPass(llvm::createModuleToFunctionPassAdaptor(std::move(FPM)));
            MPM.run(M, MAM);
        }

        // Phase 3: Post-CFF module passes (func-merge-split, indirect-branch)
        {
            llvm::ModulePassManager MPM;
            registerModulePasses(MPM, config, rng);
            MPM.run(M, MAM);
        }
    }
}

void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng) {
    FPM.addPass(InsnSubstitutionPass(config, rng));
    FPM.addPass(MBAPass(config, rng));
}

void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng) {
    // Module passes registered here as implemented
}

} // namespace cobra
