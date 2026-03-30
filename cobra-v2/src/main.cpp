#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IRReader/IRReader.h"
#include "llvm/Bitcode/BitcodeWriter.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/raw_ostream.h"

using namespace llvm;

static cl::opt<std::string> InputFile(cl::Positional,
    cl::desc("<input .bc/.ll file>"), cl::Required);
static cl::opt<std::string> OutputFile("o",
    cl::desc("Output file"), cl::value_desc("filename"), cl::init("-"));
static cl::opt<bool> EmitLL("emit-ll",
    cl::desc("Emit human-readable .ll output"), cl::init(false));

int main(int argc, char **argv) {
    cl::ParseCommandLineOptions(argc, argv, "CobraObfuscator v2\n");

    LLVMContext ctx;
    SMDiagnostic err;
    auto mod = parseIRFile(InputFile, err, ctx);
    if (!mod) {
        err.print(argv[0], errs());
        return 1;
    }

    // TODO: pass pipeline goes here

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
