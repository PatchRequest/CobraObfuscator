# CobraObfuscator v2 — LLVM IR Obfuscator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a standalone C++ CLI tool (`cobra-v2`) that reads LLVM IR (`.bc`/`.ll`), applies 10 obfuscation passes, and writes obfuscated IR back out.

**Architecture:** Standalone LLVM tool using the new pass manager. Each obfuscation pass is a `PassInfoMixin`-based class (FunctionPass or ModulePass). A `PassPipeline` class constructs the pipeline from CLI options and iterates it N times. The CLI uses `llvm::cl` for argument parsing.

**Tech Stack:** C++17, LLVM 20 (installed at `/opt/homebrew/opt/llvm`), CMake 3.20+

---

## File Map

All paths relative to `cobra-v2/` (created under project root `/Users/daniel/code/CobraObfuscator/cobra-v2/`).

| File | Responsibility |
|------|---------------|
| `CMakeLists.txt` | Build config, find LLVM, link libraries |
| `include/cobra/CobraConfig.h` | `CobraConfig` struct shared by all passes |
| `include/cobra/RNG.h` | Seeded PRNG wrapper (header-only) |
| `include/cobra/OpaquePredicates.h` | Opaque predicate builder utilities |
| `include/cobra/Passes.h` | Forward declarations + registration for all passes |
| `include/cobra/PassPipeline.h` | Pipeline construction interface |
| `src/main.cpp` | CLI entry, module load/save, invoke pipeline |
| `src/PassPipeline.cpp` | Pipeline construction, ordering, iteration |
| `src/utils/OpaquePredicates.cpp` | Opaque predicate IR generation |
| `src/passes/InsnSubstitution.cpp` | Instruction substitution pass |
| `src/passes/JunkInsertion.cpp` | Junk insertion pass |
| `src/passes/DeadCode.cpp` | Dead code / opaque predicate pass |
| `src/passes/CFF.cpp` | Control flow flattening pass |
| `src/passes/StringEncrypt.cpp` | String encryption pass |
| `src/passes/MBA.cpp` | Mixed boolean-arithmetic pass |
| `src/passes/BogusCF.cpp` | Bogus control flow pass |
| `src/passes/FuncMergeSplit.cpp` | Function merge/split pass |
| `src/passes/IndirectBranch.cpp` | Indirect branching pass |
| `src/passes/ConstantUnfold.cpp` | Constant unfolding pass |
| `test/lit.cfg.py` | Lit test driver config |
| `test/passes/*.ll` | Per-pass FileCheck tests |
| `test/e2e/*.c` + `test/e2e/run_e2e.sh` | End-to-end correctness tests |

---

## Task 1: Project Skeleton + CMake + Build Verification

**Files:**
- Create: `cobra-v2/CMakeLists.txt`
- Create: `cobra-v2/src/main.cpp`

- [ ] **Step 1: Create directory structure**

```bash
cd /Users/daniel/code/CobraObfuscator
mkdir -p cobra-v2/{src/passes,src/utils,include/cobra,test/passes,test/e2e}
```

- [ ] **Step 2: Write CMakeLists.txt**

```cmake
cmake_minimum_required(VERSION 3.20)
project(cobra-v2 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(LLVM REQUIRED CONFIG)
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
separate_arguments(LLVM_DEFINITIONS_LIST NATIVE_COMMAND ${LLVM_DEFINITIONS})
add_definitions(${LLVM_DEFINITIONS_LIST})

include_directories(${CMAKE_SOURCE_DIR}/include)

add_executable(cobra-v2
    src/main.cpp
)

llvm_map_components_to_libnames(LLVM_LIBS
    core
    irreader
    bitreader
    bitwriter
    support
    passes
)

target_link_libraries(cobra-v2 PRIVATE ${LLVM_LIBS})
```

- [ ] **Step 3: Write minimal main.cpp**

```cpp
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
```

- [ ] **Step 4: Build and verify**

```bash
cd /Users/daniel/code/CobraObfuscator/cobra-v2
cmake -B build -DLLVM_DIR=$(/opt/homebrew/opt/llvm/bin/llvm-config --cmakedir)
cmake --build build
```

Expected: builds successfully, produces `build/cobra-v2` binary.

- [ ] **Step 5: Smoke test — round-trip a .ll file**

Create `test/smoke.ll`:
```llvm
define i32 @main() {
  ret i32 0
}
```

```bash
./build/cobra-v2 test/smoke.ll -o /tmp/smoke_out.ll --emit-ll
cat /tmp/smoke_out.ll
```

Expected: outputs the same IR (no passes applied yet).

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): project skeleton with CMake + LLVM IR round-trip"
```

---

## Task 2: Config, RNG, and Pass Pipeline Infrastructure

**Files:**
- Create: `cobra-v2/include/cobra/CobraConfig.h`
- Create: `cobra-v2/include/cobra/RNG.h`
- Create: `cobra-v2/include/cobra/Passes.h`
- Create: `cobra-v2/include/cobra/PassPipeline.h`
- Create: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/src/main.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write CobraConfig.h**

```cpp
#pragma once
#include <cstdint>
#include <string>
#include <vector>

namespace cobra {

struct CobraConfig {
    uint64_t seed = 0;
    int iterations = 1;
    std::vector<std::string> enabledPasses;   // empty = all
    std::vector<std::string> excludedPasses;
    bool verbose = false;

    bool isPassEnabled(const std::string &name) const {
        if (!excludedPasses.empty()) {
            for (auto &e : excludedPasses)
                if (e == name) return false;
        }
        if (enabledPasses.empty()) return true;
        for (auto &p : enabledPasses)
            if (p == name) return true;
        return false;
    }
};

} // namespace cobra
```

- [ ] **Step 2: Write RNG.h**

```cpp
#pragma once
#include <cstdint>
#include <random>

namespace cobra {

class RNG {
public:
    explicit RNG(uint64_t seed) : engine(seed) {}

    uint64_t next() { return dist(engine); }

    uint32_t nextU32() { return static_cast<uint32_t>(next()); }

    uint32_t nextInRange(uint32_t lo, uint32_t hi) {
        return lo + (nextU32() % (hi - lo));
    }

    bool chance(double probability) {
        return (nextU32() % 10000) < static_cast<uint32_t>(probability * 10000);
    }

    RNG fork() { return RNG(next()); }

private:
    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist;
};

} // namespace cobra
```

- [ ] **Step 3: Write Passes.h (empty registrations for now)**

```cpp
#pragma once

namespace llvm {
class FunctionPassManager;
class ModulePassManager;
} // namespace llvm

namespace cobra {
struct CobraConfig;
class RNG;

void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng);
void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng);

} // namespace cobra
```

- [ ] **Step 4: Write PassPipeline.h**

```cpp
#pragma once
#include "cobra/CobraConfig.h"

namespace llvm {
class Module;
} // namespace llvm

namespace cobra {

void runPipeline(llvm::Module &M, CobraConfig &config);

} // namespace cobra
```

- [ ] **Step 5: Write PassPipeline.cpp**

```cpp
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

// Stub implementations — passes added in later tasks
void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng) {
    // Passes will be registered here as they're implemented
}

void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng) {
    // Module passes registered here as implemented
}

} // namespace cobra
```

- [ ] **Step 6: Update main.cpp to use pipeline**

Replace the `// TODO: pass pipeline goes here` line and add CLI options. Full updated `main.cpp`:

```cpp
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
```

- [ ] **Step 7: Update CMakeLists.txt to include new sources**

Replace the `add_executable` block:

```cmake
add_executable(cobra-v2
    src/main.cpp
    src/PassPipeline.cpp
)
```

- [ ] **Step 8: Build and verify**

```bash
cd /Users/daniel/code/CobraObfuscator/cobra-v2
cmake --build build
./build/cobra-v2 test/smoke.ll -o /tmp/smoke2.ll --emit-ll --verbose
```

Expected: prints `=== Iteration 1/1 ===` to stderr, outputs IR to stdout.

- [ ] **Step 9: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add config, RNG, and pass pipeline infrastructure"
```

---

## Task 3: Instruction Substitution Pass

**Files:**
- Create: `cobra-v2/src/passes/InsnSubstitution.cpp`
- Create: `cobra-v2/test/passes/insn-substitution.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/insn-substitution.ll`:

```llvm
; RUN: %cobra -o - --passes insn-substitution --seed 42 --emit-ll %s | FileCheck %s

; The add should be replaced with sub-of-neg or equivalent
; CHECK-LABEL: define i32 @test_add
; CHECK-NOT: add i32 %a, %b
; CHECK: sub i32
define i32 @test_add(i32 %a, i32 %b) {
  %r = add i32 %a, %b
  ret i32 %r
}

; The xor should be replaced with and/or chain
; CHECK-LABEL: define i32 @test_xor
; CHECK-NOT: xor i32 %a, %b
define i32 @test_xor(i32 %a, i32 %b) {
  %r = xor i32 %a, %b
  ret i32 %r
}

; The sub should be replaced
; CHECK-LABEL: define i32 @test_sub
; CHECK-NOT: sub i32 %a, %b
define i32 @test_sub(i32 %a, i32 %b) {
  %r = sub i32 %a, %b
  ret i32 %r
}
```

- [ ] **Step 2: Write lit test config**

Create `test/lit.cfg.py`:

```python
import lit.formats

config.name = "cobra-v2"
config.test_format = lit.formats.ShTest(True)
config.suffixes = ['.ll']
config.test_source_root = os.path.dirname(__file__)
config.substitutions.append(('%cobra', os.path.join(
    os.path.dirname(__file__), '..', 'build', 'cobra-v2')))
```

- [ ] **Step 3: Write InsnSubstitution.cpp**

```cpp
#include "cobra/Passes.h"
#include "cobra/CobraConfig.h"
#include "cobra/RNG.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/PassManager.h"

namespace cobra {

class InsnSubstitutionPass
    : public llvm::PassInfoMixin<InsnSubstitutionPass> {
public:
    InsnSubstitutionPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}

    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM) {
        if (!config.isPassEnabled("insn-substitution"))
            return llvm::PreservedAnalyses::all();

        bool changed = false;
        std::vector<llvm::Instruction *> worklist;
        for (auto &BB : F)
            for (auto &I : BB)
                worklist.push_back(&I);

        for (auto *I : worklist) {
            auto *bin = llvm::dyn_cast<llvm::BinaryOperator>(I);
            if (!bin) continue;

            llvm::IRBuilder<> B(bin);
            llvm::Value *replacement = nullptr;
            auto *lhs = bin->getOperand(0);
            auto *rhs = bin->getOperand(1);

            switch (bin->getOpcode()) {
            case llvm::Instruction::Add:
                // add a, b -> sub a, (sub 0, b)
                replacement = B.CreateSub(
                    lhs, B.CreateSub(
                        llvm::ConstantInt::get(rhs->getType(), 0), rhs));
                break;
            case llvm::Instruction::Sub:
                // sub a, b -> add a, (sub 0, b)
                replacement = B.CreateAdd(
                    lhs, B.CreateSub(
                        llvm::ConstantInt::get(rhs->getType(), 0), rhs));
                break;
            case llvm::Instruction::Xor:
                // xor a, b -> (a | b) & ~(a & b)
                replacement = B.CreateAnd(
                    B.CreateOr(lhs, rhs),
                    B.CreateNot(B.CreateAnd(lhs, rhs)));
                break;
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

private:
    CobraConfig &config;
    RNG &rng;
};

} // namespace cobra
```

- [ ] **Step 4: Update Passes.h with InsnSubstitutionPass declaration**

Add to `include/cobra/Passes.h` after the existing forward declarations — make it the full header with includes and the class declaration inlined (since the pass classes are defined in their own .cpp files, we register them from PassPipeline.cpp which includes both headers):

```cpp
#pragma once
#include "cobra/CobraConfig.h"
#include "cobra/RNG.h"
#include "llvm/IR/PassManager.h"

namespace cobra {

// --- Instruction Substitution ---
class InsnSubstitutionPass
    : public llvm::PassInfoMixin<InsnSubstitutionPass> {
public:
    InsnSubstitutionPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};

} // namespace cobra
```

Then in `InsnSubstitution.cpp`, remove the class definition and just implement the `run` method:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"

namespace cobra {

llvm::PreservedAnalyses InsnSubstitutionPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("insn-substitution"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    std::vector<llvm::Instruction *> worklist;
    for (auto &BB : F)
        for (auto &I : BB)
            worklist.push_back(&I);

    for (auto *I : worklist) {
        auto *bin = llvm::dyn_cast<llvm::BinaryOperator>(I);
        if (!bin) continue;

        llvm::IRBuilder<> B(bin);
        llvm::Value *replacement = nullptr;
        auto *lhs = bin->getOperand(0);
        auto *rhs = bin->getOperand(1);

        switch (bin->getOpcode()) {
        case llvm::Instruction::Add:
            replacement = B.CreateSub(
                lhs, B.CreateSub(
                    llvm::ConstantInt::get(rhs->getType(), 0), rhs));
            break;
        case llvm::Instruction::Sub:
            replacement = B.CreateAdd(
                lhs, B.CreateSub(
                    llvm::ConstantInt::get(rhs->getType(), 0), rhs));
            break;
        case llvm::Instruction::Xor:
            replacement = B.CreateAnd(
                B.CreateOr(lhs, rhs),
                B.CreateNot(B.CreateAnd(lhs, rhs)));
            break;
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
```

- [ ] **Step 5: Register pass in PassPipeline.cpp**

In `registerFunctionPasses`, add:

```cpp
void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng) {
    FPM.addPass(InsnSubstitutionPass(config, rng));
}
```

- [ ] **Step 6: Update CMakeLists.txt**

```cmake
add_executable(cobra-v2
    src/main.cpp
    src/PassPipeline.cpp
    src/passes/InsnSubstitution.cpp
)
```

- [ ] **Step 7: Build and run test**

```bash
cd /Users/daniel/code/CobraObfuscator/cobra-v2
cmake --build build
./build/cobra-v2 test/passes/insn-substitution.ll -o - --passes insn-substitution --seed 42 --emit-ll
```

Verify: output contains `sub` instead of `add`, no raw `xor` instructions remain.

- [ ] **Step 8: End-to-end correctness test**

Create `test/e2e/arith.c`:

```c
#include <stdio.h>
int main() {
    int a = 10, b = 3;
    printf("add=%d sub=%d xor=%d\n", a + b, a - b, a ^ b);
    return 0;
}
```

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_obf.bc --passes insn-substitution --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_obf.bc -o /tmp/arith_obf
/tmp/arith_obf
```

Expected output: `add=13 sub=7 xor=9`

- [ ] **Step 9: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add instruction substitution pass"
```

---

## Task 4: MBA Pass

**Files:**
- Create: `cobra-v2/src/passes/MBA.cpp`
- Create: `cobra-v2/test/passes/mba.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/mba.ll`:

```llvm
; RUN: %cobra -o - --passes mba --seed 42 --emit-ll %s | FileCheck %s

; add should become MBA: (a ^ b) + 2*(a & b)
; CHECK-LABEL: define i32 @test_mba_add
; CHECK-NOT: add i32 %a, %b
; CHECK: xor
; CHECK: and
define i32 @test_mba_add(i32 %a, i32 %b) {
  %r = add i32 %a, %b
  ret i32 %r
}

; or should become MBA: (a ^ b) + (a & b)
; CHECK-LABEL: define i32 @test_mba_or
; CHECK-NOT: or i32 %a, %b
define i32 @test_mba_or(i32 %a, i32 %b) {
  %r = or i32 %a, %b
  ret i32 %r
}
```

- [ ] **Step 2: Write MBA pass**

Add to `include/cobra/Passes.h`:

```cpp
class MBAPass : public llvm::PassInfoMixin<MBAPass> {
public:
    MBAPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/MBA.cpp`:

```cpp
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
```

- [ ] **Step 3: Register in PassPipeline.cpp**

In `registerFunctionPasses`, after InsnSubstitutionPass:

```cpp
FPM.addPass(MBAPass(config, rng));
```

- [ ] **Step 4: Update CMakeLists.txt**

Add `src/passes/MBA.cpp` to the source list.

- [ ] **Step 5: Build and test**

```bash
cmake --build build
./build/cobra-v2 test/passes/mba.ll -o - --passes mba --seed 42 --emit-ll
```

Verify: `add` replaced by xor/and/shl chain, `or` replaced by xor/and/add.

- [ ] **Step 6: E2E correctness**

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_mba.bc --passes mba --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_mba.bc -o /tmp/arith_mba
/tmp/arith_mba
```

Expected: `add=13 sub=7 xor=9`

- [ ] **Step 7: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add MBA (mixed boolean-arithmetic) pass"
```

---

## Task 5: Constant Unfolding Pass

**Files:**
- Create: `cobra-v2/src/passes/ConstantUnfold.cpp`
- Create: `cobra-v2/test/passes/constant-unfold.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/constant-unfold.ll`:

```llvm
; RUN: %cobra -o - --passes constant-unfold --seed 42 --emit-ll %s | FileCheck %s

; The constant 42 should be replaced with a runtime computation
; CHECK-LABEL: define i32 @test_const
; CHECK-NOT: ret i32 42
; CHECK: ret i32
define i32 @test_const() {
  ret i32 42
}

; Constants in arithmetic should be unfolded
; CHECK-LABEL: define i32 @test_add_const
; CHECK-NOT: add i32 %a, 100
define i32 @test_add_const(i32 %a) {
  %r = add i32 %a, 100
  ret i32 %r
}
```

- [ ] **Step 2: Write ConstantUnfold pass**

Add to `include/cobra/Passes.h`:

```cpp
class ConstantUnfoldPass : public llvm::PassInfoMixin<ConstantUnfoldPass> {
public:
    ConstantUnfoldPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
    llvm::Value *unfoldConstant(llvm::IRBuilder<> &B, llvm::ConstantInt *C);
};
```

Create `src/passes/ConstantUnfold.cpp`:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::Value *ConstantUnfoldPass::unfoldConstant(
    llvm::IRBuilder<> &B, llvm::ConstantInt *C) {
    auto *ty = C->getType();
    int64_t val = C->getSExtValue();

    // Decompose: val = a * b + c where a, b, c are random
    uint32_t a = rng.nextInRange(2, 50);
    int64_t b = val / static_cast<int64_t>(a);
    int64_t c = val - b * static_cast<int64_t>(a);

    auto *mulResult = B.CreateMul(
        llvm::ConstantInt::get(ty, a),
        llvm::ConstantInt::get(ty, b));
    return B.CreateAdd(mulResult, llvm::ConstantInt::get(ty, c));
}

llvm::PreservedAnalyses ConstantUnfoldPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("constant-unfold"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    std::vector<llvm::Instruction *> worklist;
    for (auto &BB : F)
        for (auto &I : BB)
            worklist.push_back(&I);

    for (auto *I : worklist) {
        // Skip phi nodes and switch — they need compile-time constants
        if (llvm::isa<llvm::PHINode>(I)) continue;
        if (llvm::isa<llvm::SwitchInst>(I)) continue;

        for (unsigned op = 0; op < I->getNumOperands(); ++op) {
            auto *ci = llvm::dyn_cast<llvm::ConstantInt>(I->getOperand(op));
            if (!ci) continue;
            // Don't unfold small constants (0, 1, -1) — too trivial
            int64_t val = ci->getSExtValue();
            if (val >= -1 && val <= 1) continue;
            // Don't unfold i1 (booleans)
            if (ci->getType()->getBitWidth() == 1) continue;

            llvm::IRBuilder<> B(I);
            auto *unfolded = unfoldConstant(B, ci);
            I->setOperand(op, unfolded);
            changed = true;
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp**

In `registerFunctionPasses`, add **before** InsnSubstitutionPass (constant-unfold runs first per spec ordering):

```cpp
void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng) {
    FPM.addPass(ConstantUnfoldPass(config, rng));
    FPM.addPass(InsnSubstitutionPass(config, rng));
    FPM.addPass(MBAPass(config, rng));
}
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/ConstantUnfold.cpp`. Build and run:

```bash
cmake --build build
./build/cobra-v2 test/passes/constant-unfold.ll -o - --passes constant-unfold --seed 42 --emit-ll
```

Verify: `ret i32 42` replaced with arithmetic, `add i32 %a, 100` uses computed value.

- [ ] **Step 5: E2E correctness**

```bash
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_cu.bc --passes constant-unfold --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_cu.bc -o /tmp/arith_cu
/tmp/arith_cu
```

Expected: `add=13 sub=7 xor=9`

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add constant unfolding pass"
```

---

## Task 6: Opaque Predicates Utility + Dead Code Pass

**Files:**
- Create: `cobra-v2/include/cobra/OpaquePredicates.h`
- Create: `cobra-v2/src/utils/OpaquePredicates.cpp`
- Create: `cobra-v2/src/passes/DeadCode.cpp`
- Create: `cobra-v2/test/passes/dead-code.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write OpaquePredicates.h**

```cpp
#pragma once

namespace llvm {
class Value;
class BasicBlock;
class IRBuilderBase;
} // namespace llvm

namespace cobra {
class RNG;

// Create a value that is always true at runtime but non-obvious statically.
// Inserts IR before the builder's current insertion point.
// Uses values available in the function (arguments, loads) to build
// expressions like x*(x-1)%2==0 (always true for integers).
llvm::Value *createOpaqueTrue(llvm::IRBuilderBase &B, RNG &rng);

// Create a value that is always false at runtime.
llvm::Value *createOpaqueFalse(llvm::IRBuilderBase &B, RNG &rng);

} // namespace cobra
```

- [ ] **Step 2: Write OpaquePredicates.cpp**

```cpp
#include "cobra/OpaquePredicates.h"
#include "cobra/RNG.h"

#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::Value *createOpaqueTrue(llvm::IRBuilderBase &B, RNG &rng) {
    // x * (x - 1) is always even => x*(x-1) % 2 == 0 is always true
    // Use a function argument or create a volatile load for x
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
```

- [ ] **Step 3: Write the test**

Create `test/passes/dead-code.ll`:

```llvm
; RUN: %cobra -o - --passes dead-code --seed 42 --emit-ll %s | FileCheck %s

; Should insert opaque predicate branches
; CHECK-LABEL: define i32 @test_dead
; CHECK: icmp
; CHECK: br i1
define i32 @test_dead(i32 %a) {
entry:
  %r = add i32 %a, 1
  ret i32 %r
}
```

- [ ] **Step 4: Write DeadCode pass**

Add to `include/cobra/Passes.h`:

```cpp
class DeadCodePass : public llvm::PassInfoMixin<DeadCodePass> {
public:
    DeadCodePass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/DeadCode.cpp`:

```cpp
#include "cobra/Passes.h"
#include "cobra/OpaquePredicates.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"
#include "llvm/Transforms/Utils/Cloning.h"

namespace cobra {

llvm::PreservedAnalyses DeadCodePass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("dead-code"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;

    // Collect blocks to process (skip entry — splitting it is tricky)
    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        if (&BB != &F.getEntryBlock())
            blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;
        if (!rng.chance(0.5)) continue;  // 50% of blocks

        // Create a fake block that clones this block's instructions
        // but with mutated constants
        auto *fakeBB = llvm::BasicBlock::Create(
            F.getContext(), "dead." + BB->getName(), &F);
        llvm::IRBuilder<> fakeB(fakeBB);

        // Fill fake block with junk that looks real
        auto *i32Ty = fakeB.getInt32Ty();
        auto *junk1 = fakeB.CreateAdd(
            llvm::ConstantInt::get(i32Ty, rng.nextU32()),
            llvm::ConstantInt::get(i32Ty, rng.nextU32()));
        auto *junk2 = fakeB.CreateMul(junk1,
            llvm::ConstantInt::get(i32Ty, rng.nextU32()));
        (void)junk2;
        fakeB.CreateBr(BB); // fake block jumps to real block

        // Insert opaque predicate at the start of BB's single predecessor
        // or split BB and insert before it
        auto *splitBB = BB->splitBasicBlockBefore(
            BB->begin(), "opaque." + BB->getName());

        // Replace the unconditional br in splitBB with conditional
        splitBB->getTerminator()->eraseFromParent();
        llvm::IRBuilder<> B(splitBB);
        auto *cond = createOpaqueTrue(B, rng);
        B.CreateCondBr(cond, BB, fakeBB);

        changed = true;
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 5: Register in PassPipeline.cpp**

```cpp
FPM.addPass(ConstantUnfoldPass(config, rng));
FPM.addPass(InsnSubstitutionPass(config, rng));
FPM.addPass(MBAPass(config, rng));
FPM.addPass(DeadCodePass(config, rng));
```

- [ ] **Step 6: Update CMakeLists.txt, build, test**

Add `src/passes/DeadCode.cpp` and `src/utils/OpaquePredicates.cpp`. Build and verify:

```bash
cmake --build build
./build/cobra-v2 test/passes/dead-code.ll -o - --passes dead-code --seed 42 --emit-ll
```

Verify: output has `icmp`/`br i1` guards and `dead.` blocks.

- [ ] **Step 7: E2E correctness**

```bash
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_dc.bc --passes dead-code --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_dc.bc -o /tmp/arith_dc
/tmp/arith_dc
```

Expected: `add=13 sub=7 xor=9`

- [ ] **Step 8: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add opaque predicates utility and dead code pass"
```

---

## Task 7: Bogus Control Flow Pass

**Files:**
- Create: `cobra-v2/src/passes/BogusCF.cpp`
- Create: `cobra-v2/test/passes/bogus-cf.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/bogus-cf.ll`:

```llvm
; RUN: %cobra -o - --passes bogus-cf --seed 42 --emit-ll %s | FileCheck %s

; Should add bogus conditional branches
; CHECK-LABEL: define i32 @test_bogus
; CHECK: br i1
; CHECK: bogus.
define i32 @test_bogus(i32 %a, i32 %b) {
entry:
  %r = add i32 %a, %b
  br label %exit
exit:
  ret i32 %r
}
```

- [ ] **Step 2: Write BogusCF pass**

Add to `include/cobra/Passes.h`:

```cpp
class BogusCFPass : public llvm::PassInfoMixin<BogusCFPass> {
public:
    BogusCFPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/BogusCF.cpp`:

```cpp
#include "cobra/Passes.h"
#include "cobra/OpaquePredicates.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"
#include "llvm/Transforms/Utils/Cloning.h"

namespace cobra {

llvm::PreservedAnalyses BogusCFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("bogus-cf"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;

    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;
        if (!rng.chance(0.4)) continue;  // 40% chance per block

        // Clone the block with mutated constants as the "bogus" path
        auto *bogusBB = llvm::BasicBlock::Create(
            F.getContext(), "bogus." + BB->getName(), &F);
        llvm::IRBuilder<> bogusB(bogusBB);

        // Generate fake computation in bogus block
        auto *i32Ty = bogusB.getInt32Ty();
        auto *c1 = llvm::ConstantInt::get(i32Ty, rng.nextU32());
        auto *c2 = llvm::ConstantInt::get(i32Ty, rng.nextU32());
        auto *fakeOp = bogusB.CreateXor(c1, c2);
        auto *fakeOp2 = bogusB.CreateAdd(fakeOp, c1);
        (void)fakeOp2;
        bogusB.CreateBr(BB);  // bogus path falls through to real block

        // Split before BB and insert opaque predicate
        auto *guardBB = BB->splitBasicBlockBefore(
            BB->begin(), "guard." + BB->getName());
        guardBB->getTerminator()->eraseFromParent();

        llvm::IRBuilder<> B(guardBB);
        auto *cond = createOpaqueTrue(B, rng);
        B.CreateCondBr(cond, BB, bogusBB);

        changed = true;
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp**

```cpp
FPM.addPass(ConstantUnfoldPass(config, rng));
FPM.addPass(InsnSubstitutionPass(config, rng));
FPM.addPass(MBAPass(config, rng));
FPM.addPass(BogusCFPass(config, rng));
FPM.addPass(DeadCodePass(config, rng));
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/BogusCF.cpp`. Build and run test. E2E with `arith.c` → expected `add=13 sub=7 xor=9`.

- [ ] **Step 5: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add bogus control flow pass"
```

---

## Task 8: Junk Insertion Pass

**Files:**
- Create: `cobra-v2/src/passes/JunkInsertion.cpp`
- Create: `cobra-v2/test/passes/junk-insertion.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/junk-insertion.ll`:

```llvm
; RUN: %cobra -o - --passes junk-insertion --seed 42 --emit-ll %s | FileCheck %s

; Should insert junk instructions (allocas, dead arithmetic)
; CHECK-LABEL: define i32 @test_junk
; CHECK: alloca
; CHECK: store
define i32 @test_junk(i32 %a) {
entry:
  %r = add i32 %a, 1
  ret i32 %r
}
```

- [ ] **Step 2: Write JunkInsertion pass**

Add to `include/cobra/Passes.h`:

```cpp
class JunkInsertionPass : public llvm::PassInfoMixin<JunkInsertionPass> {
public:
    JunkInsertionPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/JunkInsertion.cpp`:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses JunkInsertionPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("junk-insertion"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    bool changed = false;
    auto *i32Ty = llvm::Type::getInt32Ty(F.getContext());
    auto *i64Ty = llvm::Type::getInt64Ty(F.getContext());

    // Create junk allocas in the entry block
    auto &entry = F.getEntryBlock();
    llvm::IRBuilder<> allocaB(&entry, entry.begin());

    int numJunkVars = rng.nextInRange(2, 5);
    std::vector<llvm::AllocaInst *> junkAllocas;
    for (int i = 0; i < numJunkVars; ++i) {
        auto *ty = rng.chance(0.5) ? i32Ty : i64Ty;
        auto *alloca = allocaB.CreateAlloca(ty, nullptr, "junk");
        allocaB.CreateStore(
            llvm::ConstantInt::get(ty, rng.nextU32()), alloca);
        junkAllocas.push_back(alloca);
        changed = true;
    }

    // Scatter junk operations through each block
    std::vector<llvm::BasicBlock *> blocks;
    for (auto &BB : F)
        blocks.push_back(&BB);

    for (auto *BB : blocks) {
        if (BB->size() < 2) continue;

        // Insert 1-3 junk ops per block
        int numJunk = rng.nextInRange(1, 4);
        for (int i = 0; i < numJunk; ++i) {
            // Pick a random non-terminator instruction to insert before
            auto it = BB->begin();
            int steps = rng.nextInRange(0, std::max(1u,
                (uint32_t)BB->size() - 1));
            for (int s = 0; s < steps && it != BB->end(); ++s, ++it);
            if (it->isTerminator()) continue;

            llvm::IRBuilder<> B(&*it);

            // Pick a random junk alloca and do something useless with it
            auto *alloca = junkAllocas[rng.nextU32() % junkAllocas.size()];
            auto *ty = alloca->getAllocatedType();
            auto *loaded = B.CreateLoad(ty, alloca, /*isVolatile=*/true);
            auto *junkVal = B.CreateAdd(loaded,
                llvm::ConstantInt::get(ty, rng.nextU32()));
            B.CreateStore(junkVal, alloca, /*isVolatile=*/true);
            changed = true;
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp**

```cpp
FPM.addPass(ConstantUnfoldPass(config, rng));
FPM.addPass(InsnSubstitutionPass(config, rng));
FPM.addPass(MBAPass(config, rng));
FPM.addPass(BogusCFPass(config, rng));
FPM.addPass(DeadCodePass(config, rng));
FPM.addPass(JunkInsertionPass(config, rng));
```

- [ ] **Step 4: Update CMakeLists.txt, build, E2E test**

Add `src/passes/JunkInsertion.cpp`. Build. E2E → expected `add=13 sub=7 xor=9`.

- [ ] **Step 5: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add junk insertion pass"
```

---

## Task 9: Control Flow Flattening Pass

**Files:**
- Create: `cobra-v2/src/passes/CFF.cpp`
- Create: `cobra-v2/test/passes/cff.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/cff.ll`:

```llvm
; RUN: %cobra -o - --passes cff --seed 42 --emit-ll %s | FileCheck %s

; Should flatten control flow into a switch dispatcher
; CHECK-LABEL: define i32 @test_cff
; CHECK: switch i32
; CHECK: dispatcher
define i32 @test_cff(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 0
  br i1 %cmp, label %then, label %else

then:
  %r1 = add i32 %n, 10
  br label %merge

else:
  %r2 = sub i32 %n, 10
  br label %merge

merge:
  %r = phi i32 [ %r1, %then ], [ %r2, %else ]
  ret i32 %r
}
```

- [ ] **Step 2: Write CFF pass**

Add to `include/cobra/Passes.h`:

```cpp
class CFFPass : public llvm::PassInfoMixin<CFFPass> {
public:
    CFFPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/CFF.cpp`:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"
#include "llvm/Transforms/Utils/LowerSwitch.h"

#include <algorithm>
#include <vector>

namespace cobra {

llvm::PreservedAnalyses CFFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("cff"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();

    // Need at least 3 blocks to be worth flattening
    if (F.size() < 3) return llvm::PreservedAnalyses::all();

    auto &ctx = F.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);

    // Collect all blocks except entry
    std::vector<llvm::BasicBlock *> origBlocks;
    for (auto &BB : F)
        origBlocks.push_back(&BB);

    auto *entryBB = origBlocks[0];

    // Separate entry block: split it so the entry only sets up the
    // state variable and jumps to the dispatcher
    // Find the first non-alloca, non-phi instruction to split at
    auto splitPoint = entryBB->begin();
    while (splitPoint != entryBB->end() &&
           (llvm::isa<llvm::AllocaInst>(&*splitPoint))) {
        ++splitPoint;
    }

    llvm::BasicBlock *firstBB = entryBB->splitBasicBlock(
        splitPoint, "cff.first");

    // Create state variable alloca in entry
    llvm::IRBuilder<> entryB(entryBB->getTerminator());
    auto *stateVar = entryB.CreateAlloca(i32Ty, nullptr, "cff.state");

    // Rebuild block list (entry is now just allocas + br, firstBB has
    // the real code)
    std::vector<llvm::BasicBlock *> flatBlocks;
    for (auto &BB : F) {
        if (&BB == entryBB) continue;  // skip entry
        flatBlocks.push_back(&BB);
    }

    // Assign random state IDs to each block
    std::vector<uint32_t> stateIDs;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        stateIDs.push_back(rng.nextU32());

    // Create dispatcher block
    auto *dispatchBB = llvm::BasicBlock::Create(ctx, "cff.dispatcher", &F);
    llvm::IRBuilder<> dispB(dispatchBB);
    auto *stateVal = dispB.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *defaultBB = llvm::BasicBlock::Create(ctx, "cff.default", &F);
    llvm::IRBuilder<>(defaultBB).CreateUnreachable();
    auto *sw = dispB.CreateSwitch(stateVal, defaultBB, flatBlocks.size());

    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        sw->addCase(llvm::ConstantInt::get(i32Ty, stateIDs[i]),
                     flatBlocks[i]);
    }

    // Set initial state in entry block
    entryBB->getTerminator()->eraseFromParent();
    llvm::IRBuilder<> entryB2(entryBB);
    entryB2.CreateStore(
        llvm::ConstantInt::get(i32Ty, stateIDs[0]), stateVar);
    entryB2.CreateBr(dispatchBB);

    // Build a map from block to state ID
    std::map<llvm::BasicBlock *, uint32_t> blockToState;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        blockToState[flatBlocks[i]] = stateIDs[i];

    // Replace terminators: instead of branching to successors,
    // set state variable and branch to dispatcher
    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        auto *BB = flatBlocks[i];
        auto *term = BB->getTerminator();
        if (!term) continue;

        if (auto *br = llvm::dyn_cast<llvm::BranchInst>(term)) {
            if (br->isUnconditional()) {
                auto *dest = br->getSuccessor(0);
                auto it = blockToState.find(dest);
                if (it != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    B.CreateStore(
                        llvm::ConstantInt::get(i32Ty, it->second),
                        stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            } else {
                // Conditional branch
                auto *cond = br->getCondition();
                auto *trueDest = br->getSuccessor(0);
                auto *falseDest = br->getSuccessor(1);
                auto trueIt = blockToState.find(trueDest);
                auto falseIt = blockToState.find(falseDest);

                if (trueIt != blockToState.end() &&
                    falseIt != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    auto *trueState = llvm::ConstantInt::get(
                        i32Ty, trueIt->second);
                    auto *falseState = llvm::ConstantInt::get(
                        i32Ty, falseIt->second);
                    auto *selected = B.CreateSelect(
                        cond, trueState, falseState);
                    B.CreateStore(selected, stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            }
        } else if (auto *ret = llvm::dyn_cast<llvm::ReturnInst>(term)) {
            // Return instructions stay as-is
            (void)ret;
        }
    }

    // Fix PHI nodes: PHIs that referenced original predecessors now
    // need to reference the dispatcher block (since all paths come
    // through it). We demote PHIs to allocas to avoid this complexity.
    std::vector<llvm::PHINode *> phis;
    for (auto *BB : flatBlocks)
        for (auto &I : *BB)
            if (auto *phi = llvm::dyn_cast<llvm::PHINode>(&I))
                phis.push_back(phi);

    for (auto *phi : phis) {
        // Create alloca in entry for each phi
        llvm::IRBuilder<> allocB(entryBB, entryBB->begin());
        auto *alloca = allocB.CreateAlloca(phi->getType(), nullptr,
                                            phi->getName() + ".demoted");

        // For each incoming value, store to alloca before the branch
        // to dispatcher in that block
        for (unsigned j = 0; j < phi->getNumIncomingValues(); ++j) {
            auto *val = phi->getIncomingValue(j);
            auto *pred = phi->getIncomingBlock(j);
            // Find the store-to-statevar + br-to-dispatcher in pred
            auto *predTerm = pred->getTerminator();
            llvm::IRBuilder<> B(predTerm);
            B.CreateStore(val, alloca);
        }

        // Replace phi with load from alloca
        llvm::IRBuilder<> B(phi->getParent(), phi->getParent()->begin());
        // Skip past any remaining phis
        while (llvm::isa<llvm::PHINode>(&*B.GetInsertPoint()))
            B.SetInsertPoint(&*std::next(B.GetInsertPoint()));
        auto *loaded = B.CreateLoad(phi->getType(), alloca);
        phi->replaceAllUsesWith(loaded);
        phi->eraseFromParent();
    }

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp**

```cpp
FPM.addPass(ConstantUnfoldPass(config, rng));
FPM.addPass(InsnSubstitutionPass(config, rng));
FPM.addPass(MBAPass(config, rng));
FPM.addPass(BogusCFPass(config, rng));
FPM.addPass(DeadCodePass(config, rng));
FPM.addPass(JunkInsertionPass(config, rng));
FPM.addPass(CFFPass(config, rng));
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/CFF.cpp`. Build. Check `--emit-ll` output for `switch i32` dispatcher.

- [ ] **Step 5: E2E correctness test**

Create `test/e2e/controlflow.c`:

```c
#include <stdio.h>
int classify(int n) {
    if (n > 0) return 1;
    else if (n < 0) return -1;
    else return 0;
}
int main() {
    printf("pos=%d neg=%d zero=%d\n", classify(5), classify(-3), classify(0));
    return 0;
}
```

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/controlflow.c -o /tmp/cf.bc
./build/cobra-v2 /tmp/cf.bc -o /tmp/cf_obf.bc --passes cff --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/cf_obf.bc -o /tmp/cf_obf
/tmp/cf_obf
```

Expected: `pos=1 neg=-1 zero=0`

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add control flow flattening pass"
```

---

## Task 10: String Encryption Pass

**Files:**
- Create: `cobra-v2/src/passes/StringEncrypt.cpp`
- Create: `cobra-v2/test/passes/string-encrypt.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/string-encrypt.ll`:

```llvm
; RUN: %cobra -o - --passes string-encrypt --seed 42 --emit-ll %s | FileCheck %s

; The string constant should be replaced with encrypted bytes
; CHECK-NOT: c"Hello World\00"
; CHECK: @cobra.enc.
@.str = private unnamed_addr constant [12 x i8] c"Hello World\00"

declare i32 @puts(ptr)

define i32 @main() {
  %1 = call i32 @puts(ptr @.str)
  ret i32 0
}
```

- [ ] **Step 2: Write StringEncrypt pass**

Add to `include/cobra/Passes.h`:

```cpp
class StringEncryptPass : public llvm::PassInfoMixin<StringEncryptPass> {
public:
    StringEncryptPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/StringEncrypt.cpp`:

```cpp
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
    bool changed = false;

    // Collect string globals
    std::vector<llvm::GlobalVariable *> stringGlobals;
    for (auto &GV : M.globals()) {
        if (!GV.hasInitializer()) continue;
        if (!GV.isConstant()) continue;
        auto *init = llvm::dyn_cast<llvm::ConstantDataArray>(
            GV.getInitializer());
        if (!init) continue;
        if (!init->isString() && !init->isCString()) continue;
        stringGlobals.push_back(&GV);
    }

    for (auto *GV : stringGlobals) {
        auto *init = llvm::cast<llvm::ConstantDataArray>(
            GV->getInitializer());
        auto strData = init->getAsString();
        size_t len = strData.size();

        // Generate XOR key
        std::vector<uint8_t> key(len);
        for (size_t i = 0; i < len; ++i)
            key[i] = static_cast<uint8_t>(rng.nextU32());

        // Encrypt the string
        std::vector<uint8_t> encrypted(len);
        for (size_t i = 0; i < len; ++i)
            encrypted[i] = static_cast<uint8_t>(strData[i]) ^ key[i];

        // Create encrypted global
        auto *encType = llvm::ArrayType::get(i8Ty, len);
        auto *encInit = llvm::ConstantDataArray::get(ctx, encrypted);
        auto *encGV = new llvm::GlobalVariable(
            M, encType, false, llvm::GlobalValue::PrivateLinkage,
            encInit, "cobra.enc." + GV->getName());

        // Create key global
        auto *keyInit = llvm::ConstantDataArray::get(ctx, key);
        auto *keyGV = new llvm::GlobalVariable(
            M, encType, true, llvm::GlobalValue::PrivateLinkage,
            keyInit, "cobra.key." + GV->getName());

        // Create a decryption function
        auto *decFnTy = llvm::FunctionType::get(
            llvm::PointerType::getUnqual(ctx), {}, false);
        auto *decFn = llvm::Function::Create(
            decFnTy, llvm::GlobalValue::PrivateLinkage,
            "cobra.dec." + GV->getName(), M);

        auto *entryBB = llvm::BasicBlock::Create(ctx, "entry", decFn);
        llvm::IRBuilder<> B(entryBB);

        // Loop: for each byte, XOR encrypted with key
        auto *i64Ty = B.getInt64Ty();
        auto *loopBB = llvm::BasicBlock::Create(ctx, "loop", decFn);
        auto *exitBB = llvm::BasicBlock::Create(ctx, "exit", decFn);

        B.CreateBr(loopBB);

        // Loop header
        B.SetInsertPoint(loopBB);
        auto *idx = B.CreatePHI(i64Ty, 2, "idx");
        idx->addIncoming(llvm::ConstantInt::get(i64Ty, 0), entryBB);

        auto *encPtr = B.CreateGEP(i8Ty, encGV,
            {llvm::ConstantInt::get(i64Ty, 0), idx});
        auto *keyPtr = B.CreateGEP(i8Ty, keyGV,
            {llvm::ConstantInt::get(i64Ty, 0), idx});
        auto *encByte = B.CreateLoad(i8Ty, encPtr);
        auto *keyByte = B.CreateLoad(i8Ty, keyPtr);
        auto *decByte = B.CreateXor(encByte, keyByte);
        B.CreateStore(decByte, encPtr);

        auto *nextIdx = B.CreateAdd(idx, llvm::ConstantInt::get(i64Ty, 1));
        idx->addIncoming(nextIdx, loopBB);
        auto *done = B.CreateICmpEQ(nextIdx,
            llvm::ConstantInt::get(i64Ty, len));
        B.CreateCondBr(done, exitBB, loopBB);

        // Return pointer to decrypted data (in-place in encGV)
        B.SetInsertPoint(exitBB);
        B.CreateRet(encGV);

        // Replace all uses of original string: at each use site,
        // insert a call to the decryption function
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
                // Handle constant expression GEP users
                std::vector<llvm::Use *> ceUses;
                for (auto &ceUse : ce->uses())
                    ceUses.push_back(&ceUse);
                for (auto *ceUse : ceUses) {
                    if (auto *inst = llvm::dyn_cast<llvm::Instruction>(
                            ceUse->getUser())) {
                        llvm::IRBuilder<> useB(inst);
                        auto *decrypted = useB.CreateCall(decFn);
                        ceUse->set(decrypted);
                    }
                }
            }
        }

        changed = true;
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp Phase 1**

In the Phase 1 section of `runPipeline`:

```cpp
// Phase 1: Pre-CFF module passes
{
    llvm::ModulePassManager MPM;
    MPM.addPass(StringEncryptPass(config, rng));
    MPM.run(M, MAM);
}
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/StringEncrypt.cpp`. Build and verify IR test.

- [ ] **Step 5: E2E correctness**

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_se.bc --passes string-encrypt --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_se.bc -o /tmp/arith_se
/tmp/arith_se
```

Expected: `add=13 sub=7 xor=9` (format string is decrypted at runtime).

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add string encryption pass"
```

---

## Task 11: Function Merge/Split Pass

**Files:**
- Create: `cobra-v2/src/passes/FuncMergeSplit.cpp`
- Create: `cobra-v2/test/passes/func-merge-split.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/func-merge-split.ll`:

```llvm
; RUN: %cobra -o - --passes func-merge-split --seed 42 --emit-ll %s | FileCheck %s

; Two simple functions should get merged into one dispatcher
; CHECK: switch i32
; CHECK: cobra.merged
define i32 @foo(i32 %a) {
  %r = add i32 %a, 10
  ret i32 %r
}

define i32 @bar(i32 %a) {
  %r = mul i32 %a, 2
  ret i32 %r
}

define i32 @main() {
  %a = call i32 @foo(i32 5)
  %b = call i32 @bar(i32 3)
  %r = add i32 %a, %b
  ret i32 %r
}
```

- [ ] **Step 2: Write FuncMergeSplit pass**

Add to `include/cobra/Passes.h`:

```cpp
class FuncMergeSplitPass
    : public llvm::PassInfoMixin<FuncMergeSplitPass> {
public:
    FuncMergeSplitPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/FuncMergeSplit.cpp`:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Constants.h"

namespace cobra {

llvm::PreservedAnalyses FuncMergeSplitPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("func-merge-split"))
        return llvm::PreservedAnalyses::all();

    auto &ctx = M.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);
    bool changed = false;

    // Collect candidate pairs: internal functions with same return type
    // and same parameter types
    std::vector<llvm::Function *> candidates;
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        if (F.hasExternalLinkage() && F.getName() == "main") continue;
        if (F.isVarArg()) continue;
        if (F.getName().starts_with("cobra.")) continue;
        candidates.push_back(&F);
    }

    // Merge pairs of functions with identical signatures
    std::vector<bool> merged(candidates.size(), false);
    for (size_t i = 0; i < candidates.size(); ++i) {
        if (merged[i]) continue;
        for (size_t j = i + 1; j < candidates.size(); ++j) {
            if (merged[j]) continue;
            auto *f1 = candidates[i];
            auto *f2 = candidates[j];

            // Must have same function type
            if (f1->getFunctionType() != f2->getFunctionType()) continue;

            if (!rng.chance(0.5)) continue;

            // Create merged function with extra i32 selector param
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
            auto *defaultBB = llvm::BasicBlock::Create(
                ctx, "default", mergedFn);

            // Entry: switch on selector
            llvm::IRBuilder<> entryB(entryBB);
            auto argIt = mergedFn->arg_begin();
            llvm::Value *selector = &*argIt;
            auto *sw = entryB.CreateSwitch(selector, defaultBB, 2);
            sw->addCase(llvm::ConstantInt::get(i32Ty, 0), f1BB);
            sw->addCase(llvm::ConstantInt::get(i32Ty, 1), f2BB);

            // Default: unreachable
            llvm::IRBuilder<>(defaultBB).CreateUnreachable();

            // Collect the forwarded args (skip selector)
            std::vector<llvm::Value *> fwdArgs;
            ++argIt;
            for (; argIt != mergedFn->arg_end(); ++argIt)
                fwdArgs.push_back(&*argIt);

            // Case f1: call f1 with args, return result
            {
                llvm::IRBuilder<> B(f1BB);
                auto *result = B.CreateCall(f1, fwdArgs);
                B.CreateRet(result);
            }

            // Case f2: call f2 with args, return result
            {
                llvm::IRBuilder<> B(f2BB);
                auto *result = B.CreateCall(f2, fwdArgs);
                B.CreateRet(result);
            }

            // Replace call sites of f1 and f2
            auto replaceCallSites = [&](llvm::Function *origFn,
                                         uint32_t selectorVal) {
                std::vector<llvm::CallInst *> calls;
                for (auto &use : origFn->uses()) {
                    if (auto *call = llvm::dyn_cast<llvm::CallInst>(
                            use.getUser())) {
                        if (call->getCalledFunction() == origFn)
                            calls.push_back(call);
                    }
                }
                for (auto *call : calls) {
                    llvm::IRBuilder<> B(call);
                    std::vector<llvm::Value *> args;
                    args.push_back(
                        llvm::ConstantInt::get(i32Ty, selectorVal));
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
            break; // move to next unmerged function
        }
    }

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 3: Register in PassPipeline.cpp Phase 3**

```cpp
// Phase 3: Post-CFF module passes
{
    llvm::ModulePassManager MPM;
    MPM.addPass(FuncMergeSplitPass(config, rng));
    MPM.run(M, MAM);
}
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/FuncMergeSplit.cpp`. Build. Verify IR output contains `cobra.merged` and `switch i32`.

- [ ] **Step 5: E2E correctness**

Using the `func-merge-split.ll` test compiled to binary, `main` should return `(5+10) + (3*2) = 21`.

```bash
./build/cobra-v2 test/passes/func-merge-split.ll -o /tmp/fms.bc --passes func-merge-split --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/fms.bc -o /tmp/fms
/tmp/fms; echo $?
```

Expected exit code: 21

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add function merge/split pass"
```

---

## Task 12: Indirect Branch Pass

**Files:**
- Create: `cobra-v2/src/passes/IndirectBranch.cpp`
- Create: `cobra-v2/test/passes/indirect-branch.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/indirect-branch.ll`:

```llvm
; RUN: %cobra -o - --passes indirect-branch --seed 42 --emit-ll %s | FileCheck %s

; Direct calls should be replaced with indirect calls via table
; CHECK: @cobra.fptable
; CHECK-NOT: call i32 @target(
define i32 @target(i32 %a) {
  %r = add i32 %a, 1
  ret i32 %r
}

define i32 @caller(i32 %x) {
  %r = call i32 @target(i32 %x)
  ret i32 %r
}
```

- [ ] **Step 2: Write IndirectBranch pass**

Add to `include/cobra/Passes.h`:

```cpp
class IndirectBranchPass
    : public llvm::PassInfoMixin<IndirectBranchPass> {
public:
    IndirectBranchPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

Create `src/passes/IndirectBranch.cpp`:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Constants.h"

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

    // Create global function pointer table
    auto *tableTy = llvm::ArrayType::get(ptrTy, targets.size());
    std::vector<llvm::Constant *> tableEntries;
    for (auto *F : targets)
        tableEntries.push_back(F);

    auto *tableInit = llvm::ConstantArray::get(tableTy, tableEntries);
    auto *table = new llvm::GlobalVariable(
        M, tableTy, true, llvm::GlobalValue::PrivateLinkage,
        tableInit, "cobra.fptable");

    // Replace direct calls with indirect calls through the table
    auto *i64Ty = llvm::Type::getInt64Ty(ctx);

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
                if (!rng.chance(0.7)) continue; // 70% of calls
                calls.push_back(call);
            }

            for (auto *call : calls) {
                auto *callee = call->getCalledFunction();
                size_t idx = funcToIdx[callee];

                llvm::IRBuilder<> B(call);
                auto *gep = B.CreateGEP(ptrTy, table, {
                    llvm::ConstantInt::get(i64Ty, 0),
                    llvm::ConstantInt::get(i64Ty, idx)
                });
                auto *fp = B.CreateLoad(ptrTy, gep);

                std::vector<llvm::Value *> args;
                for (unsigned i = 0; i < call->arg_size(); ++i)
                    args.push_back(call->getArgOperand(i));

                auto *newCall = B.CreateCall(
                    callee->getFunctionType(), fp, args);
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
```

- [ ] **Step 3: Register in PassPipeline.cpp Phase 3**

```cpp
// Phase 3: Post-CFF module passes
{
    llvm::ModulePassManager MPM;
    MPM.addPass(FuncMergeSplitPass(config, rng));
    MPM.addPass(IndirectBranchPass(config, rng));
    MPM.run(M, MAM);
}
```

- [ ] **Step 4: Update CMakeLists.txt, build, test**

Add `src/passes/IndirectBranch.cpp`. Build. Verify `@cobra.fptable` in output.

- [ ] **Step 5: E2E correctness**

```bash
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_ib.bc --passes indirect-branch --seed 42
/opt/homebrew/opt/llvm/bin/clang /tmp/arith_ib.bc -o /tmp/arith_ib
/tmp/arith_ib
```

Expected: `add=13 sub=7 xor=9`

- [ ] **Step 6: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add indirect branch pass"
```

---

## Task 13: Full Pipeline Integration + E2E Test Suite

**Files:**
- Create: `cobra-v2/test/e2e/run_e2e.sh`
- Create: `cobra-v2/test/e2e/strings.c`
- Create: `cobra-v2/test/e2e/recursion.c`
- Modify: `cobra-v2/src/PassPipeline.cpp` (final pass ordering verification)

- [ ] **Step 1: Create comprehensive E2E test programs**

Create `test/e2e/strings.c`:

```c
#include <stdio.h>
#include <string.h>

int main() {
    const char *hello = "Hello";
    const char *world = "World";
    char buf[20];
    snprintf(buf, sizeof(buf), "%s %s", hello, world);
    printf("combined=%s len=%zu\n", buf, strlen(buf));
    return 0;
}
```

Create `test/e2e/recursion.c`:

```c
#include <stdio.h>

int fib(int n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    printf("fib(10)=%d\n", fib(10));
    return 0;
}
```

- [ ] **Step 2: Create E2E test runner**

Create `test/e2e/run_e2e.sh`:

```bash
#!/bin/bash
set -e

COBRA="$(dirname "$0")/../../build/cobra-v2"
CLANG="/opt/homebrew/opt/llvm/bin/clang"
PASS=0
FAIL=0

run_test() {
    local name="$1"
    local src="$2"
    local expected="$3"
    local passes="${4:-all}"

    echo -n "  $name ($passes)... "

    $CLANG -emit-llvm -c "$src" -o /tmp/cobra_e2e_${name}.bc 2>/dev/null
    $COBRA /tmp/cobra_e2e_${name}.bc -o /tmp/cobra_e2e_${name}_obf.bc \
        --passes "$passes" --seed 42 2>/dev/null
    $CLANG /tmp/cobra_e2e_${name}_obf.bc -o /tmp/cobra_e2e_${name} 2>/dev/null

    local actual
    actual=$(/tmp/cobra_e2e_${name} 2>&1)

    if [ "$actual" = "$expected" ]; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL"
        echo "    expected: $expected"
        echo "    actual:   $actual"
        FAIL=$((FAIL + 1))
    fi
}

echo "Running E2E tests..."

TESTDIR="$(dirname "$0")"

# Individual passes
run_test "arith_insn" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "insn-substitution"
run_test "arith_mba" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "mba"
run_test "arith_cu" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "constant-unfold"
run_test "arith_dc" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "dead-code"
run_test "arith_bcf" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "bogus-cf"
run_test "arith_junk" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "junk-insertion"
run_test "arith_cff" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "cff"
run_test "arith_se" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "string-encrypt"

# Control flow
run_test "cf_cff" "$TESTDIR/controlflow.c" "pos=1 neg=-1 zero=0" "cff"

# Strings
run_test "strings_se" "$TESTDIR/strings.c" "combined=Hello World len=11" "string-encrypt"

# Recursion
run_test "recursion_cff" "$TESTDIR/recursion.c" "fib(10)=55" "cff"

# All passes combined
run_test "arith_all" "$TESTDIR/arith.c" "add=13 sub=7 xor=9" "all"
run_test "cf_all" "$TESTDIR/controlflow.c" "pos=1 neg=-1 zero=0" "all"
run_test "strings_all" "$TESTDIR/strings.c" "combined=Hello World len=11" "all"
run_test "recursion_all" "$TESTDIR/recursion.c" "fib(10)=55" "all"

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
```

- [ ] **Step 3: Make it executable and run**

```bash
chmod +x test/e2e/run_e2e.sh
cd /Users/daniel/code/CobraObfuscator/cobra-v2
./test/e2e/run_e2e.sh
```

Expected: all tests pass. Fix any failing tests.

- [ ] **Step 4: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add full E2E test suite for all passes"
```

---

## Task 14: Final Cleanup + Verbose Output

**Files:**
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/src/main.cpp`

- [ ] **Step 1: Add verbose per-pass logging**

In each pass's `run()` method, at the end before returning, if `config.verbose` is true, print stats to `llvm::errs()`. Example for InsnSubstitution:

```cpp
if (config.verbose && changed)
    llvm::errs() << "[insn-substitution] " << F.getName()
                 << ": substituted instructions\n";
```

Add similar lines to every pass.

- [ ] **Step 2: Add --version flag to main.cpp**

Before `cl::ParseCommandLineOptions`:

```cpp
cl::SetVersionPrinter([](raw_ostream &OS) {
    OS << "CobraObfuscator v2.0.0\n";
});
```

- [ ] **Step 3: Build and run with --verbose**

```bash
cmake --build build
./build/cobra-v2 /tmp/arith.bc -o /tmp/out.bc --verbose --seed 42
```

Expected: verbose output showing which passes ran on which functions.

- [ ] **Step 4: Final E2E run**

```bash
./test/e2e/run_e2e.sh
```

All tests must pass.

- [ ] **Step 5: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add verbose logging and version flag"
```
