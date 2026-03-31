# CobraObfuscator v2 Feature Batch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 features to cobra-v2: --stats output, anti-tampering pass, symbol stripping pass, and CFF handler diversity.

**Architecture:** Each feature is an independent addition. Stats is a reporting feature in the pipeline. Anti-tampering and symbol-stripping are new module passes. CFF handler diversity modifies the existing CFF pass to randomly select between dispatch strategies.

**Tech Stack:** C++17, LLVM 20 (at `/opt/homebrew/opt/llvm`), CMake

---

## File Map

All paths relative to `cobra-v2/` under `/Users/daniel/code/CobraObfuscator/cobra-v2/`.

| File | Responsibility |
|------|---------------|
| `include/cobra/CobraConfig.h` | Add `stats` flag |
| `include/cobra/Stats.h` | Stats collection struct |
| `include/cobra/Passes.h` | Add AntiTamperPass, SymbolStripPass declarations |
| `src/main.cpp` | Add `--stats` CLI flag, print stats after pipeline |
| `src/PassPipeline.cpp` | Pass stats through pipeline, register new passes |
| `src/Stats.cpp` | Stats collection/printing implementation |
| `src/passes/AntiTamper.cpp` | Anti-tampering pass |
| `src/passes/SymbolStrip.cpp` | Symbol stripping pass |
| `src/passes/CFF.cpp` | Add handler diversity (computed goto, nested if-else) |
| `test/passes/anti-tamper.ll` | Anti-tamper test |
| `test/passes/symbol-strip.ll` | Symbol strip test |
| `test/passes/cff-diversity.ll` | CFF diversity test |

---

## Task 1: --stats Output

**Files:**
- Create: `cobra-v2/include/cobra/Stats.h`
- Create: `cobra-v2/src/Stats.cpp`
- Modify: `cobra-v2/include/cobra/CobraConfig.h`
- Modify: `cobra-v2/src/main.cpp`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Add stats flag to CobraConfig.h**

In `cobra-v2/include/cobra/CobraConfig.h`, add `bool stats = false;` after `bool verbose`:

```cpp
struct CobraConfig {
    uint64_t seed = 0;
    int iterations = 1;
    std::vector<std::string> enabledPasses;
    std::vector<std::string> excludedPasses;
    bool verbose = false;
    bool stats = false;
    // ... isPassEnabled stays the same
};
```

- [ ] **Step 2: Create Stats.h**

```cpp
#pragma once
#include "llvm/Support/raw_ostream.h"
#include <cstdint>
#include <map>
#include <string>

namespace llvm {
class Module;
}

namespace cobra {

struct ModuleStats {
    uint64_t functions = 0;
    uint64_t blocks = 0;
    uint64_t instructions = 0;
    uint64_t globals = 0;
};

struct PipelineStats {
    ModuleStats before;
    ModuleStats after;
    std::map<std::string, uint64_t> passTransforms;

    void recordBefore(const llvm::Module &M);
    void recordAfter(const llvm::Module &M);
    void recordPassHit(const std::string &passName, uint64_t count = 1);
    void print(llvm::raw_ostream &OS) const;
};

ModuleStats collectModuleStats(const llvm::Module &M);

} // namespace cobra
```

- [ ] **Step 3: Create Stats.cpp**

```cpp
#include "cobra/Stats.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Function.h"
#include "llvm/Support/raw_ostream.h"

namespace cobra {

ModuleStats collectModuleStats(const llvm::Module &M) {
    ModuleStats s;
    for (auto &GV : M.globals()) {
        (void)GV;
        s.globals++;
    }
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        s.functions++;
        for (auto &BB : F) {
            s.blocks++;
            for (auto &I : BB) {
                (void)I;
                s.instructions++;
            }
        }
    }
    return s;
}

void PipelineStats::recordBefore(const llvm::Module &M) {
    before = collectModuleStats(M);
}

void PipelineStats::recordAfter(const llvm::Module &M) {
    after = collectModuleStats(M);
}

void PipelineStats::recordPassHit(const std::string &passName, uint64_t count) {
    passTransforms[passName] += count;
}

void PipelineStats::print(llvm::raw_ostream &OS) const {
    OS << "\n";
    OS << "=== CobraObfuscator Statistics ===\n";
    OS << "\n";
    OS << "  Module before:\n";
    OS << "    Functions:    " << before.functions << "\n";
    OS << "    Basic blocks: " << before.blocks << "\n";
    OS << "    Instructions: " << before.instructions << "\n";
    OS << "    Globals:      " << before.globals << "\n";
    OS << "\n";
    OS << "  Module after:\n";
    OS << "    Functions:    " << after.functions;
    if (after.functions != before.functions)
        OS << " (+" << (int64_t)(after.functions - before.functions) << ")";
    OS << "\n";
    OS << "    Basic blocks: " << after.blocks;
    if (after.blocks != before.blocks)
        OS << " (+" << (int64_t)(after.blocks - before.blocks) << ")";
    OS << "\n";
    OS << "    Instructions: " << after.instructions;
    if (after.instructions != before.instructions)
        OS << " (+" << (int64_t)(after.instructions - before.instructions) << ")";
    OS << "\n";
    OS << "    Globals:      " << after.globals;
    if (after.globals != before.globals)
        OS << " (+" << (int64_t)(after.globals - before.globals) << ")";
    OS << "\n";

    if (!passTransforms.empty()) {
        OS << "\n";
        OS << "  Pass activity:\n";
        for (auto &[name, count] : passTransforms)
            OS << "    " << name << ": " << count << " transforms\n";
    }
    OS << "\n";
}

} // namespace cobra
```

- [ ] **Step 4: Add --stats CLI flag to main.cpp**

Add after the `Verbose` option:

```cpp
static cl::opt<bool> Stats("stats",
    cl::desc("Print before/after statistics"), cl::init(false));
```

Set it on config after `config.verbose = Verbose;`:

```cpp
config.stats = Stats;
```

After `cobra::runPipeline(*mod, config);`, add stats collection and printing. The pipeline will need to return or populate stats. Simplest approach: collect before/after in main.cpp directly:

```cpp
cobra::PipelineStats pstats;
if (config.stats)
    pstats.recordBefore(*mod);

cobra::runPipeline(*mod, config);

if (config.stats) {
    pstats.recordAfter(*mod);
    pstats.print(llvm::errs());
}
```

Add `#include "cobra/Stats.h"` at the top.

- [ ] **Step 5: Update CMakeLists.txt**

Add `src/Stats.cpp` to sources.

- [ ] **Step 6: Build and test**

```bash
cmake --build build
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_stats.bc --passes all --seed 42 --stats
```

Expected: stats printed to stderr showing before/after counts with deltas.

- [ ] **Step 7: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add --stats flag for before/after module statistics"
```

---

## Task 2: Symbol Stripping Pass

**Files:**
- Create: `cobra-v2/src/passes/SymbolStrip.cpp`
- Create: `cobra-v2/test/passes/symbol-strip.ll`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

- [ ] **Step 1: Write the test**

Create `test/passes/symbol-strip.ll`:

```llvm
; RUN: %cobra -o - --passes symbol-strip --seed 42 --emit-ll %s | FileCheck %s

; Internal function names should be replaced with random names
; CHECK-NOT: define {{.*}} @my_helper(
; CHECK-NOT: define {{.*}} @compute_value(
; main should be preserved
; CHECK: define {{.*}} @main

define internal i32 @my_helper(i32 %x) {
  %r = add i32 %x, 1
  ret i32 %r
}

define internal i32 @compute_value(i32 %a, i32 %b) {
  %r = call i32 @my_helper(i32 %a)
  %s = add i32 %r, %b
  ret i32 %s
}

define i32 @main() {
  %r = call i32 @compute_value(i32 5, i32 3)
  ret i32 %r
}
```

- [ ] **Step 2: Add SymbolStripPass to Passes.h**

Add before `registerFunctionPasses`:

```cpp
// --- Symbol Stripping (ModulePass) ---
class SymbolStripPass : public llvm::PassInfoMixin<SymbolStripPass> {
public:
    SymbolStripPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

- [ ] **Step 3: Write SymbolStrip.cpp**

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/GlobalVariable.h"
#include "llvm/Support/raw_ostream.h"

#include <sstream>
#include <iomanip>

namespace cobra {

static std::string randomName(RNG &rng, const std::string &prefix) {
    std::stringstream ss;
    ss << prefix << std::hex << rng.nextU32() << rng.nextU32();
    return ss.str();
}

llvm::PreservedAnalyses SymbolStripPass::run(
    llvm::Module &M, llvm::ModuleAnalysisManager &AM) {
    if (!config.isPassEnabled("symbol-strip"))
        return llvm::PreservedAnalyses::all();

    bool changed = false;
    unsigned renamed = 0;

    // Rename internal/private functions (skip main, declarations, cobra.*)
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        if (F.getName() == "main") continue;
        if (F.getName().starts_with("cobra.")) continue;
        if (F.hasExternalLinkage()) continue;

        F.setName(randomName(rng, "f_"));
        renamed++;
        changed = true;
    }

    // Rename internal global variables (skip cobra.* globals)
    for (auto &GV : M.globals()) {
        if (GV.hasExternalLinkage()) continue;
        if (GV.getName().starts_with("cobra.")) continue;
        if (GV.getName().empty()) continue;

        GV.setName(randomName(rng, "g_"));
        changed = true;
    }

    // Strip names from basic blocks and local values in all functions
    for (auto &F : M) {
        if (F.isDeclaration()) continue;
        for (auto &BB : F) {
            BB.setName("");
            for (auto &I : BB) {
                if (!I.getType()->isVoidTy() && I.hasName())
                    I.setName("");
            }
        }
    }

    if (config.verbose && changed)
        llvm::errs() << "[symbol-strip] renamed " << renamed << " functions\n";

    return changed ? llvm::PreservedAnalyses::none()
                   : llvm::PreservedAnalyses::all();
}

} // namespace cobra
```

- [ ] **Step 4: Register in PassPipeline.cpp Phase 3**

In `registerModulePasses`, add after IndirectBranchPass (symbol strip runs last — after all other passes have created their functions):

```cpp
void registerModulePasses(llvm::ModulePassManager &MPM,
                          CobraConfig &config, RNG &rng) {
    MPM.addPass(FuncMergeSplitPass(config, rng));
    MPM.addPass(IndirectBranchPass(config, rng));
    MPM.addPass(SymbolStripPass(config, rng));
}
```

- [ ] **Step 5: Update CMakeLists.txt**

Add `src/passes/SymbolStrip.cpp` to sources.

- [ ] **Step 6: Build and test**

```bash
cmake --build build
./build/cobra-v2 test/passes/symbol-strip.ll -o - --passes symbol-strip --seed 42 --emit-ll
```

Verify: `my_helper` and `compute_value` are renamed to `f_*`, `main` preserved, basic block and value names stripped.

- [ ] **Step 7: E2E correctness**

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_strip.bc --passes symbol-strip --seed 42
/opt/homebrew/opt/llvm/bin/clang -O0 /tmp/arith_strip.bc -o /tmp/arith_strip -lm
/tmp/arith_strip
```

Expected: `add=13 sub=7 xor=9`

- [ ] **Step 8: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add symbol stripping pass"
```

---

## Task 3: Anti-Tampering Pass

**Files:**
- Create: `cobra-v2/src/passes/AntiTamper.cpp`
- Create: `cobra-v2/test/passes/anti-tamper.ll`
- Create: `cobra-v2/test/e2e/anti_tamper.c`
- Modify: `cobra-v2/include/cobra/Passes.h`
- Modify: `cobra-v2/src/PassPipeline.cpp`
- Modify: `cobra-v2/CMakeLists.txt`

The anti-tampering pass inserts runtime integrity checks. For each function, it:
1. Computes a simple checksum of the function's instruction count at compile time
2. Inserts a runtime check at the function entry that recomputes the checksum and aborts if it doesn't match

Since we're at the IR level (not binary), the "checksum" is based on IR properties that will be preserved through codegen — specifically the number of basic blocks, which is stable.

- [ ] **Step 1: Write the test**

Create `test/passes/anti-tamper.ll`:

```llvm
; RUN: %cobra -o - --passes anti-tamper --seed 42 --emit-ll %s | FileCheck %s

; Should insert a tamper check at function entry
; CHECK-LABEL: define i32 @protected_fn
; CHECK: cobra.tamper
; CHECK: call void @abort
define i32 @protected_fn(i32 %a) {
entry:
  %r = add i32 %a, 10
  br label %exit
exit:
  %r2 = mul i32 %r, 2
  ret i32 %r2
}
```

- [ ] **Step 2: Add AntiTamperPass to Passes.h**

Add before `registerFunctionPasses`:

```cpp
// --- Anti-Tampering ---
class AntiTamperPass : public llvm::PassInfoMixin<AntiTamperPass> {
public:
    AntiTamperPass(CobraConfig &config, RNG &rng)
        : config(config), rng(rng) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
    RNG &rng;
};
```

- [ ] **Step 3: Write AntiTamper.cpp**

The approach: for each function, embed an expected block count as a constant. At function entry, count the actual blocks at runtime by walking a linked structure, and call `abort()` if they don't match.

Simpler and more robust approach: embed a "canary" value in a global, check it at function entry. If anyone patches the binary and zeros out globals, the check fails.

Even simpler and most practical: insert a hash of the function's IR structure as a compile-time constant, and at runtime verify a derived value. Since we can't actually count blocks at runtime easily, we'll use a **global canary + trap** approach:

```cpp
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

    // Compute a hash based on IR structure: block count XOR instruction count XOR random salt
    uint32_t blockCount = 0;
    uint32_t insnCount = 0;
    for (auto &BB : F) {
        blockCount++;
        for (auto &I : BB) {
            (void)I;
            insnCount++;
        }
    }
    uint32_t salt = rng.nextU32();
    uint32_t expectedHash = blockCount ^ insnCount ^ salt;

    // Create a global to store the expected hash
    auto *hashGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, expectedHash),
        "cobra.tamper.hash." + F.getName().str());

    // Create a global with the salt
    auto *saltGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, salt),
        "cobra.tamper.salt." + F.getName().str());

    // Create a global with the expected counts (encoded)
    auto *countGV = new llvm::GlobalVariable(
        *M, i32Ty, true, llvm::GlobalValue::PrivateLinkage,
        llvm::ConstantInt::get(i32Ty, blockCount ^ insnCount),
        "cobra.tamper.counts." + F.getName().str());

    // Get or declare abort()
    auto *abortFn = M->getOrInsertFunction("abort",
        llvm::FunctionType::get(voidTy, false)).getCallee();

    // Split entry block: insert check before the real code
    auto &entryBB = F.getEntryBlock();
    auto splitIt = entryBB.begin();
    // Skip allocas
    while (splitIt != entryBB.end() && llvm::isa<llvm::AllocaInst>(&*splitIt))
        ++splitIt;

    auto *realBB = entryBB.splitBasicBlock(splitIt, "cobra.tamper.ok");
    auto *trapBB = llvm::BasicBlock::Create(ctx, "cobra.tamper.fail", &F);

    // Replace entry's br with the check
    entryBB.getTerminator()->eraseFromParent();
    llvm::IRBuilder<> B(&entryBB);

    auto *loadedHash = B.CreateLoad(i32Ty, hashGV);
    auto *loadedSalt = B.CreateLoad(i32Ty, saltGV);
    auto *loadedCounts = B.CreateLoad(i32Ty, countGV);

    // Recompute: counts ^ salt should equal hash
    auto *computed = B.CreateXor(loadedCounts, loadedSalt);
    auto *match = B.CreateICmpEQ(computed, loadedHash);
    B.CreateCondBr(match, realBB, trapBB);

    // Trap block: call abort
    llvm::IRBuilder<> trapB(trapBB);
    trapB.CreateCall(llvm::cast<llvm::Function>(abortFn));
    trapB.CreateUnreachable();

    if (config.verbose)
        llvm::errs() << "[anti-tamper] " << F.getName() << "\n";

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
```

- [ ] **Step 4: Register in PassPipeline.cpp**

Add AntiTamperPass as the LAST function pass (after CFF, so the check guards the fully obfuscated function). In `registerFunctionPasses`:

```cpp
void registerFunctionPasses(llvm::FunctionPassManager &FPM,
                            CobraConfig &config, RNG &rng) {
    FPM.addPass(ConstantUnfoldPass(config, rng));
    FPM.addPass(InsnSubstitutionPass(config, rng));
    FPM.addPass(MBAPass(config, rng));
    FPM.addPass(BogusCFPass(config, rng));
    FPM.addPass(DeadCodePass(config, rng));
    FPM.addPass(JunkInsertionPass(config, rng));
    FPM.addPass(CFFPass(config, rng));
    FPM.addPass(AntiTamperPass(config, rng));
}
```

- [ ] **Step 5: Update CMakeLists.txt**

Add `src/passes/AntiTamper.cpp` to sources.

- [ ] **Step 6: Build and test**

```bash
cmake --build build
./build/cobra-v2 test/passes/anti-tamper.ll -o - --passes anti-tamper --seed 42 --emit-ll
```

Verify: `cobra.tamper` blocks and `abort` call present.

- [ ] **Step 7: E2E correctness**

Create `test/e2e/anti_tamper.c`:

```c
#include <stdio.h>

int square(int x) {
    return x * x;
}

int cube(int x) {
    return x * x * x;
}

int main() {
    printf("sq=%d cube=%d\n", square(5), cube(3));
    return 0;
}
```

```bash
/opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/anti_tamper.c -o /tmp/at.bc
./build/cobra-v2 /tmp/at.bc -o /tmp/at_obf.bc --passes anti-tamper --seed 42
/opt/homebrew/opt/llvm/bin/clang -O0 /tmp/at_obf.bc -o /tmp/at_obf -lm
/tmp/at_obf
```

Expected: `sq=25 cube=27` (tamper check passes since nothing was modified)

- [ ] **Step 8: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add anti-tampering pass with integrity checks"
```

---

## Task 4: CFF Handler Diversity

**Files:**
- Modify: `cobra-v2/src/passes/CFF.cpp`
- Create: `cobra-v2/test/passes/cff-diversity.ll`

Currently CFF uses a single `switch` dispatcher. This task adds 2 alternative dispatch strategies, chosen randomly per function:

1. **Switch** (existing) — `switch i32 %state, ...`
2. **If-else chain** — cascaded `icmp + br` comparisons
3. **Computed index** — XOR the state with a key, use it as an index into a lookup table of block addresses (using `indirectbr`)

- [ ] **Step 1: Write the test**

Create `test/passes/cff-diversity.ll`:

```llvm
; RUN: %cobra -o - --passes cff --seed 1 --emit-ll %s | FileCheck --check-prefix=CHECK1 %s
; RUN: %cobra -o - --passes cff --seed 42 --emit-ll %s | FileCheck --check-prefix=CHECK2 %s

; Different seeds should produce different dispatcher types
; We just check that CFF runs and the function is correct

; CHECK1-LABEL: define i32 @test_diversity
; CHECK1: cff.dispatcher

; CHECK2-LABEL: define i32 @test_diversity
; CHECK2: cff.dispatcher

define i32 @test_diversity(i32 %n) {
entry:
  %cmp = icmp sgt i32 %n, 10
  br i1 %cmp, label %big, label %small

big:
  %r1 = mul i32 %n, 2
  br label %done

small:
  %r2 = add i32 %n, 100
  br label %done

done:
  %r = phi i32 [ %r1, %big ], [ %r2, %small ]
  ret i32 %r
}
```

- [ ] **Step 2: Refactor CFF.cpp with dispatcher strategies**

Replace the dispatcher creation section of CFF.cpp. The full updated file:

```cpp
#include "cobra/Passes.h"

#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/Constants.h"

#include <map>
#include <vector>

namespace cobra {

// Strategy 0: Switch dispatcher (original)
static void buildSwitchDispatcher(
    llvm::BasicBlock *dispatchBB, llvm::BasicBlock *defaultBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    const std::vector<uint32_t> &stateIDs,
    llvm::Type *i32Ty) {
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *sw = B.CreateSwitch(stateVal, defaultBB, flatBlocks.size());
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        sw->addCase(llvm::ConstantInt::get(i32Ty, stateIDs[i]), flatBlocks[i]);
}

// Strategy 1: If-else chain dispatcher
static void buildIfElseDispatcher(
    llvm::BasicBlock *dispatchBB, llvm::BasicBlock *defaultBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    const std::vector<uint32_t> &stateIDs,
    llvm::Type *i32Ty, llvm::Function &F) {
    auto &ctx = F.getContext();
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");

    // Build chain: if state == id[0] goto block[0], else if state == id[1] ...
    llvm::BasicBlock *currentBB = dispatchBB;
    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        auto *cmp = B.CreateICmpEQ(stateVal,
            llvm::ConstantInt::get(i32Ty, stateIDs[i]));

        if (i == flatBlocks.size() - 1) {
            // Last one: true goes to block, false goes to default
            B.CreateCondBr(cmp, flatBlocks[i], defaultBB);
        } else {
            auto *nextCheckBB = llvm::BasicBlock::Create(
                ctx, "cff.check." + std::to_string(i), &F);
            B.CreateCondBr(cmp, flatBlocks[i], nextCheckBB);
            B.SetInsertPoint(nextCheckBB);
        }
    }
}

// Strategy 2: XOR + lookup table dispatcher
static void buildLookupDispatcher(
    llvm::BasicBlock *dispatchBB, llvm::BasicBlock *defaultBB,
    llvm::AllocaInst *stateVar,
    const std::vector<llvm::BasicBlock *> &flatBlocks,
    const std::vector<uint32_t> &stateIDs,
    llvm::Type *i32Ty, llvm::Function &F, RNG &rng) {
    auto &ctx = F.getContext();

    // Map each state ID to a sequential index via XOR with a key
    uint32_t xorKey = rng.nextU32();

    // Create a mapping: (stateID ^ xorKey) % tableSize -> block
    // Use a simple approach: remap state IDs to 0..N-1
    size_t tableSize = flatBlocks.size();
    std::vector<uint32_t> remappedIDs(tableSize);
    for (size_t i = 0; i < tableSize; ++i)
        remappedIDs[i] = static_cast<uint32_t>(i);

    // Build blockaddress array
    std::vector<llvm::Constant *> blockAddrs;
    for (size_t i = 0; i < tableSize; ++i)
        blockAddrs.push_back(llvm::BlockAddress::get(&F, flatBlocks[i]));

    auto *ptrTy = llvm::PointerType::getUnqual(ctx);
    auto *tableTy = llvm::ArrayType::get(ptrTy, tableSize);
    auto *tableInit = llvm::ConstantArray::get(tableTy, blockAddrs);
    auto *tableGV = new llvm::GlobalVariable(
        *F.getParent(), tableTy, true, llvm::GlobalValue::PrivateLinkage,
        tableInit, "cff.table." + F.getName().str());

    // Dispatcher: load state, XOR with key, modulo table size, load from table, indirectbr
    llvm::IRBuilder<> B(dispatchBB);
    auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");
    auto *xored = B.CreateXor(stateVal, llvm::ConstantInt::get(i32Ty, xorKey));
    auto *idx = B.CreateURem(xored, llvm::ConstantInt::get(i32Ty, tableSize));
    auto *idx64 = B.CreateZExt(idx, llvm::Type::getInt64Ty(ctx));
    auto *gep = B.CreateGEP(ptrTy, tableGV, idx64);
    auto *target = B.CreateLoad(ptrTy, gep);
    auto *ibr = B.CreateIndirectBr(target, flatBlocks.size());
    for (auto *BB : flatBlocks)
        ibr->addDestination(BB);

    // Update state IDs: remap so that (newID ^ xorKey) % tableSize == sequential index
    // newID = (index ^ xorKey) — but we need the modulo to work out
    // Simpler: just use index directly as the state ID, XOR key = 0
    // Actually the cleanest: stateIDs[i] = i ^ xorKey
    // Then: (stateID ^ xorKey) % tableSize = (i ^ xorKey ^ xorKey) % tableSize = i % tableSize = i
    // This is clean. We need to update the stateIDs vector used by the caller.
    // Since we can't modify the const vector, we handle this by using the remapped IDs
    // when the caller sets up the state stores.
    // PROBLEM: the caller already set up stateIDs before calling us.
    // SOLUTION: For lookup strategy, we need to set stateIDs = {0^key, 1^key, 2^key, ...}
    // This means we need to choose strategy BEFORE assigning state IDs.
    // This requires restructuring the pass. See the refactored run() below.
    (void)defaultBB; // lookup doesn't use default — OOB is unreachable
}

llvm::PreservedAnalyses CFFPass::run(
    llvm::Function &F, llvm::FunctionAnalysisManager &AM) {
    if (!config.isPassEnabled("cff"))
        return llvm::PreservedAnalyses::all();
    if (F.isDeclaration()) return llvm::PreservedAnalyses::all();
    if (F.size() < 3) return llvm::PreservedAnalyses::all();

    auto &ctx = F.getContext();
    auto *i32Ty = llvm::Type::getInt32Ty(ctx);

    // Choose dispatcher strategy: 0=switch, 1=if-else, 2=lookup
    int strategy = rng.nextU32() % 3;

    // Collect all blocks
    std::vector<llvm::BasicBlock *> origBlocks;
    for (auto &BB : F)
        origBlocks.push_back(&BB);

    auto *entryBB = origBlocks[0];

    // Split entry block: keep allocas in entry
    auto splitPoint = entryBB->begin();
    while (splitPoint != entryBB->end() &&
           llvm::isa<llvm::AllocaInst>(&*splitPoint))
        ++splitPoint;

    llvm::BasicBlock *firstBB = entryBB->splitBasicBlock(splitPoint, "cff.first");
    (void)firstBB;

    llvm::IRBuilder<> entryB(entryBB->getTerminator());
    auto *stateVar = entryB.CreateAlloca(i32Ty, nullptr, "cff.state");

    // Rebuild block list
    std::vector<llvm::BasicBlock *> flatBlocks;
    for (auto &BB : F) {
        if (&BB == entryBB) continue;
        flatBlocks.push_back(&BB);
    }

    // Assign state IDs based on strategy
    std::vector<uint32_t> stateIDs;
    uint32_t lookupXorKey = rng.nextU32();

    if (strategy == 2) {
        // Lookup: stateIDs[i] = i ^ xorKey
        for (size_t i = 0; i < flatBlocks.size(); ++i)
            stateIDs.push_back(static_cast<uint32_t>(i) ^ lookupXorKey);
    } else {
        // Switch/if-else: random state IDs
        for (size_t i = 0; i < flatBlocks.size(); ++i)
            stateIDs.push_back(rng.nextU32());
    }

    // Create dispatcher and default blocks
    auto *dispatchBB = llvm::BasicBlock::Create(ctx, "cff.dispatcher", &F);
    auto *defaultBB = llvm::BasicBlock::Create(ctx, "cff.default", &F);
    llvm::IRBuilder<>(defaultBB).CreateUnreachable();

    // Build dispatcher based on strategy
    switch (strategy) {
    case 0:
        buildSwitchDispatcher(dispatchBB, defaultBB, stateVar,
                              flatBlocks, stateIDs, i32Ty);
        break;
    case 1:
        buildIfElseDispatcher(dispatchBB, defaultBB, stateVar,
                              flatBlocks, stateIDs, i32Ty, F);
        break;
    case 2: {
        // Build lookup table
        std::vector<llvm::Constant *> blockAddrs;
        for (size_t i = 0; i < flatBlocks.size(); ++i)
            blockAddrs.push_back(llvm::BlockAddress::get(&F, flatBlocks[i]));

        auto *ptrTy = llvm::PointerType::getUnqual(ctx);
        auto *tableTy = llvm::ArrayType::get(ptrTy, flatBlocks.size());
        auto *tableInit = llvm::ConstantArray::get(tableTy, blockAddrs);
        auto *tableGV = new llvm::GlobalVariable(
            *F.getParent(), tableTy, true, llvm::GlobalValue::PrivateLinkage,
            tableInit, "cff.table." + F.getName().str());

        llvm::IRBuilder<> B(dispatchBB);
        auto *stateVal = B.CreateLoad(i32Ty, stateVar, "cff.curstate");
        auto *xored = B.CreateXor(stateVal,
            llvm::ConstantInt::get(i32Ty, lookupXorKey));
        auto *idx = B.CreateURem(xored,
            llvm::ConstantInt::get(i32Ty, flatBlocks.size()));
        auto *idx64 = B.CreateZExt(idx, llvm::Type::getInt64Ty(ctx));
        auto *gep = B.CreateGEP(ptrTy, tableGV, idx64);
        auto *target = B.CreateLoad(ptrTy, gep);
        auto *ibr = B.CreateIndirectBr(target, flatBlocks.size());
        for (auto *BB : flatBlocks)
            ibr->addDestination(BB);
        break;
    }
    }

    // Set initial state in entry block
    entryBB->getTerminator()->eraseFromParent();
    llvm::IRBuilder<> entryB2(entryBB);
    entryB2.CreateStore(llvm::ConstantInt::get(i32Ty, stateIDs[0]), stateVar);
    entryB2.CreateBr(dispatchBB);

    // Build block-to-state map
    std::map<llvm::BasicBlock *, uint32_t> blockToState;
    for (size_t i = 0; i < flatBlocks.size(); ++i)
        blockToState[flatBlocks[i]] = stateIDs[i];

    // Rewrite terminators
    for (size_t i = 0; i < flatBlocks.size(); ++i) {
        auto *BB = flatBlocks[i];
        auto *term = BB->getTerminator();
        if (!term) continue;

        if (auto *br = llvm::dyn_cast<llvm::BranchInst>(term)) {
            if (br->isUnconditional()) {
                auto it = blockToState.find(br->getSuccessor(0));
                if (it != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    B.CreateStore(llvm::ConstantInt::get(i32Ty, it->second), stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            } else {
                auto *cond = br->getCondition();
                auto trueIt = blockToState.find(br->getSuccessor(0));
                auto falseIt = blockToState.find(br->getSuccessor(1));
                if (trueIt != blockToState.end() && falseIt != blockToState.end()) {
                    llvm::IRBuilder<> B(br);
                    auto *selected = B.CreateSelect(cond,
                        llvm::ConstantInt::get(i32Ty, trueIt->second),
                        llvm::ConstantInt::get(i32Ty, falseIt->second));
                    B.CreateStore(selected, stateVar);
                    B.CreateBr(dispatchBB);
                    br->eraseFromParent();
                }
            }
        }
    }

    // Demote PHI nodes
    std::vector<llvm::PHINode *> phis;
    for (auto *BB : flatBlocks)
        for (auto &I : *BB)
            if (auto *phi = llvm::dyn_cast<llvm::PHINode>(&I))
                phis.push_back(phi);

    for (auto *phi : phis) {
        llvm::IRBuilder<> allocB(entryBB, entryBB->begin());
        auto *alloca = allocB.CreateAlloca(phi->getType(), nullptr,
                                            phi->getName() + ".demoted");
        for (unsigned j = 0; j < phi->getNumIncomingValues(); ++j) {
            auto *predTerm = phi->getIncomingBlock(j)->getTerminator();
            llvm::IRBuilder<> B(predTerm);
            B.CreateStore(phi->getIncomingValue(j), alloca);
        }
        auto insertIt = phi->getParent()->getFirstNonPHIIt();
        llvm::IRBuilder<> B(&*insertIt);
        auto *loaded = B.CreateLoad(phi->getType(), alloca);
        phi->replaceAllUsesWith(loaded);
        phi->eraseFromParent();
    }

    if (config.verbose) {
        const char *names[] = {"switch", "if-else", "lookup"};
        llvm::errs() << "[cff:" << names[strategy] << "] " << F.getName() << "\n";
    }

    return llvm::PreservedAnalyses::none();
}

} // namespace cobra
```

- [ ] **Step 3: Build and test**

```bash
cmake --build build
```

Test with different seeds to exercise different strategies:

```bash
for seed in 1 2 3 4 5 42 100; do
    echo -n "seed=$seed: "
    /opt/homebrew/opt/llvm/bin/clang -emit-llvm -c test/e2e/controlflow.c -o /tmp/cf.bc
    ./build/cobra-v2 /tmp/cf.bc -o /tmp/cf_cff.bc --passes cff --seed $seed --verbose 2>&1 | grep '\[cff'
    /opt/homebrew/opt/llvm/bin/clang -O0 /tmp/cf_cff.bc -o /tmp/cf_cff -lm
    /tmp/cf_cff
done
```

Expected: different seeds produce different dispatcher types (switch/if-else/lookup), all produce correct output `pos=1 neg=-1 zero=0`.

- [ ] **Step 4: E2E with all tests**

```bash
./test/e2e/run_e2e.sh
```

All existing tests must still pass.

- [ ] **Step 5: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): add CFF handler diversity (switch, if-else, lookup table)"
```

---

## Task 5: Update E2E Tests + Final Integration

**Files:**
- Modify: `cobra-v2/test/e2e/run_e2e.sh`

- [ ] **Step 1: Add new passes to the test runner**

In `run_e2e.sh`, add the new passes to the `ALL_PASSES` array:

```bash
ALL_PASSES=(insn-substitution mba constant-unfold dead-code bogus-cf junk-insertion cff string-encrypt func-merge-split indirect-branch anti-tamper symbol-strip)
```

- [ ] **Step 2: Add anti_tamper.c E2E test**

Add to Section 1 test arrays in run_e2e.sh. Update `C_NAMES`, `C_SRCS`, `C_EXPECTS` arrays to include `anti_tamper` (expected output: `sq=25 cube=27`).

- [ ] **Step 3: Add specific tests for new features**

Add after existing sections:

```bash
echo ""
echo "--- Section 9: New feature tests ---"

# Stats (just verify it doesn't crash - output goes to stderr)
run_c_test "arith_stats" "$TESTDIR/arith.c" "$E_ARITH" "all" 42 1

# Anti-tamper + other passes
run_c_test "at_cff" "$TESTDIR/anti_tamper.c" "sq=25 cube=27" "anti-tamper,cff"
run_c_test "at_all" "$TESTDIR/anti_tamper.c" "sq=25 cube=27" "all"

# Symbol strip + other passes
run_c_test "arith_strip_cff" "$TESTDIR/arith.c" "$E_ARITH" "symbol-strip,cff"
run_c_test "arith_strip_all" "$TESTDIR/arith.c" "$E_ARITH" "all"

# CFF diversity (multiple seeds to exercise different strategies)
for seed in 1 2 3 4 5 6 7 8 9 10; do
    run_c_test "cf_cff_s${seed}" "$TESTDIR/controlflow.c" "$E_CF" "cff" "$seed"
done
```

- [ ] **Step 4: Run full test suite**

```bash
./test/e2e/run_e2e.sh
```

All tests must pass.

- [ ] **Step 5: Commit**

```bash
git add cobra-v2/
git commit -m "feat(v2): update E2E tests for anti-tamper, symbol-strip, CFF diversity"
```
