// LLVM Pass Plugin entry point for CobraObfuscator v2
// Usage:
//   clang -fpass-plugin=libcobra.dylib foo.c -o foo
//   clang -fpass-plugin=libcobra.dylib -mllvm -cobra-seed=42 -mllvm -cobra-passes=cff,mba foo.c -o foo
//   RUSTFLAGS="-C llvm-args=-load-pass-plugin=/path/to/libcobra.dylib" cargo build

#include "cobra/PassPipeline.h"
#include "cobra/CobraConfig.h"
#include "cobra/Passes.h"
#include "cobra/RNG.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/PassManager.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"

#include <cstdlib>
#include <random>

// Plugin configuration via environment variables:
//   COBRA_SEED=42          RNG seed (0 or unset = random)
//   COBRA_ITERATIONS=2     Pipeline iterations
//   COBRA_PASSES=cff,mba   Comma-separated pass list (unset = all)
//   COBRA_EXCLUDE=junk      Passes to exclude
//   COBRA_VERBOSE=1         Verbose output

namespace {

static std::vector<std::string> splitComma(const std::string &s) {
    std::vector<std::string> result;
    if (s.empty() || s == "all") return result;
    size_t start = 0;
    while (start < s.size()) {
        auto end = s.find(',', start);
        if (end == std::string::npos) end = s.size();
        result.push_back(s.substr(start, end - start));
        start = end + 1;
    }
    return result;
}

// Wrapper ModulePass that runs the entire cobra pipeline
class CobraObfuscatorPass
    : public llvm::PassInfoMixin<CobraObfuscatorPass> {
public:
    static std::string getEnv(const char *name, const char *def = "") {
        auto *v = std::getenv(name);
        return v ? v : def;
    }

    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &MAM) {
        cobra::CobraConfig config;
        auto seedStr = getEnv("COBRA_SEED", "0");
        uint64_t seed = std::stoull(seedStr);
        config.seed = seed == 0 ? std::random_device{}() : seed;
        config.iterations = std::stoi(getEnv("COBRA_ITERATIONS", "1"));
        config.enabledPasses = splitComma(getEnv("COBRA_PASSES", "all"));
        config.excludedPasses = splitComma(getEnv("COBRA_EXCLUDE", ""));
        config.verbose = getEnv("COBRA_VERBOSE") == "1";

        cobra::runPipeline(M, config);

        return llvm::PreservedAnalyses::none();
    }
};

} // anonymous namespace

// Plugin registration
llvm::PassPluginLibraryInfo getCobraPluginInfo() {
    return {LLVM_PLUGIN_API_VERSION, "CobraObfuscator", "2.0.0",
            [](llvm::PassBuilder &PB) {
                // Register as an optimizer last EP — runs after all
                // standard optimizations
                PB.registerOptimizerLastEPCallback(
                    [](llvm::ModulePassManager &MPM,
                       llvm::OptimizationLevel,
                       llvm::ThinOrFullLTOPhase) {
                        MPM.addPass(CobraObfuscatorPass());
                    });

                // Also allow explicit pipeline specification
                PB.registerPipelineParsingCallback(
                    [](llvm::StringRef Name, llvm::ModulePassManager &MPM,
                       llvm::ArrayRef<llvm::PassBuilder::PipelineElement>) {
                        if (Name == "cobra-obfuscate") {
                            MPM.addPass(CobraObfuscatorPass());
                            return true;
                        }
                        return false;
                    });
            }};
}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
    return getCobraPluginInfo();
}
