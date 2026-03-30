#include "cobra/PassPipeline.h"
#include "cobra/CobraConfig.h"

#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IRReader/IRReader.h"
#include "llvm/Bitcode/BitcodeWriter.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/raw_ostream.h"

#include <random>

using namespace llvm;

static cl::opt<std::string> InputFile(cl::Positional,
    cl::desc("<input .bc/.ll file>"), cl::Required);
static cl::opt<std::string> OutputFile("o",
    cl::desc("Output file"), cl::value_desc("filename"), cl::init("-"));
static cl::opt<bool> EmitLL("emit-ll",
    cl::desc("Emit human-readable .ll output"), cl::init(false));
static cl::opt<std::string> Passes("passes",
    cl::desc("Comma-separated pass names or 'all'"), cl::init("all"));
static cl::opt<std::string> Exclude("exclude",
    cl::desc("Comma-separated passes to skip"), cl::init(""));
static cl::opt<uint64_t> Seed("seed",
    cl::desc("RNG seed"), cl::init(0));
static cl::opt<int> Iterations("iterations",
    cl::desc("Pipeline iterations"), cl::init(1));
static cl::opt<bool> Verbose("verbose",
    cl::desc("Print per-pass statistics"), cl::init(false));

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

int main(int argc, char **argv) {
    cl::ParseCommandLineOptions(argc, argv, "CobraObfuscator v2\n");

    LLVMContext ctx;
    SMDiagnostic err;
    auto mod = parseIRFile(InputFile, err, ctx);
    if (!mod) {
        err.print(argv[0], errs());
        return 1;
    }

    cobra::CobraConfig config;
    config.seed = Seed == 0
        ? std::random_device{}()
        : static_cast<uint64_t>(Seed);
    config.iterations = Iterations;
    config.enabledPasses = splitComma(Passes);
    config.excludedPasses = splitComma(Exclude);
    config.verbose = Verbose;

    cobra::runPipeline(*mod, config);

    std::error_code ec;
    raw_fd_ostream out(OutputFile, ec, sys::fs::OF_None);
    if (ec) {
        errs() << "Error opening output: " << ec.message() << "\n";
        return 1;
    }

    if (EmitLL)
        mod->print(out, nullptr);
    else
        WriteBitcodeToFile(*mod, out);

    return 0;
}
