# CobraObfuscator v2 — LLVM IR Obfuscator

## Overview

Standalone C++ CLI tool that transforms LLVM IR (`.bc`/`.ll`) into obfuscated LLVM IR. Language-agnostic — works with any LLVM-based compiler (Clang, Rustc, etc.). Slots into any build pipeline between IR emission and final compilation.

This replaces the binary-to-binary approach of v1 with an IR-level approach that avoids relocation issues, jump table problems, calling convention mismatches, and all the other pain points of post-link binary rewriting.

## Usage

```bash
# C workflow
clang -emit-llvm -c foo.c -o foo.bc
cobra-v2 foo.bc -o foo_obf.bc --passes all --seed 42 --iterations 2
clang foo_obf.bc -o foo

# Rust workflow
rustc --emit=llvm-bc foo.rs -o foo.bc
cobra-v2 foo.bc -o foo_obf.bc
clang foo_obf.bc -o foo

# Selective passes
cobra-v2 foo.bc -o foo_obf.bc --passes cff,mba,string-encrypt

# Exclude specific passes
cobra-v2 foo.bc -o foo_obf.bc --passes all --exclude func-merge-split
```

## Tech Stack

- **Language**: C++17
- **Build system**: CMake 3.20+
- **LLVM**: 17+ (stable PassBuilder API, new pass manager)
- **No other external dependencies** — LLVM provides everything (IR manipulation, CLI parsing via `llvm::cl`, file I/O)

## Passes

### Ported from v1 (4 passes)

#### 1. `insn-substitution` (FunctionPass)
Replace IR operations with semantically equivalent alternatives.
- `add a, b` → `sub a, (sub 0, b)`
- `xor a, b` → `(a | b) & ~(a & b)` (via select/and/or chains)
- `sub a, b` → `add a, (sub 0, b)`
- `mul a, b` → shift+add decomposition for power-of-2 cases
- Randomly chosen from multiple equivalent forms per operation.

#### 2. `junk-insertion` (FunctionPass)
Insert dead IR instructions with no observable side effects.
- Dead stores to local `alloca`s that are never loaded
- Unused arithmetic on existing SSA values (result unused)
- Redundant `bitcast`/`inttoptr`/`ptrtoint` chains that go nowhere
- These survive at IR level but many will be cleaned up by the backend unless they have subtle dependencies. Use `optnone` attribute or volatile semantics where needed to prevent backend elimination.

#### 3. `dead-code` (FunctionPass)
Opaque predicates guarding unreachable blocks.
- Build predicates from runtime values that are always true/false but non-obvious (e.g., `x * (x - 1) % 2 == 0` is always true for integers)
- Insert conditional branches using these predicates
- The "unreachable" side contains plausible-looking code cloned/mutated from real blocks
- Leverages SSA type info to generate more convincing fake code than v1's binary-level approach.

#### 4. `cff` — Control Flow Flattening (FunctionPass)
Dispatcher-based flattening operating directly on basic blocks.
- Collect all basic blocks in a function
- Create a dispatcher block with a `switch` on a state variable (`alloca i32`)
- Each original block sets the state variable to the next block's ID before jumping back to the dispatcher
- Conditional branches become state variable assignments based on the condition
- No relocation issues, no stack frame aliasing — the IR handles it natively.

### New IR-level passes (6 passes)

#### 5. `string-encrypt` (ModulePass)
Encrypt global string constants, decrypt at runtime.
- Scan module for `@.str`-style global constants with string initializers
- Replace each with an encrypted byte array (XOR with per-string random key)
- Generate a decryption stub: either a `__attribute__((constructor))` function that decrypts all strings at startup, or inline decryption at each use site
- Default to inline decryption (harder to find a single decryption routine to patch out)
- Key storage: embed keys as additional global byte arrays with obfuscated names.

#### 6. `mba` — Mixed Boolean-Arithmetic (FunctionPass)
Replace arithmetic operations with equivalent opaque expressions.
- `a + b` → `(a ^ b) + 2 * (a & b)`
- `a - b` → `(a ^ b) - 2 * (~a & b)`
- `a ^ b` → `(a | b) - (a & b)`
- `a | b` → `(a ^ b) + (a & b)` (alternative: `(a & ~b) + b`)
- Nest transformations: apply MBA to sub-expressions of MBA results for deeper obfuscation
- Nesting depth controlled by iteration count.

#### 7. `bogus-cf` — Bogus Control Flow (FunctionPass)
Insert fake conditional branches using opaque predicates built from runtime values.
- Before each basic block, insert a conditional branch with an opaque predicate
- The "false" path leads to a cloned but subtly mutated copy of the real block (different constants, swapped operands)
- Both paths converge at the original successor via phi nodes
- Opaque predicates use properties of runtime values: `(x | 1) != 0` (always true), `x^2 >= 0` (always true for unsigned), quadratic residue checks
- More robust than v1 because we have full type information and SSA form.

#### 8. `func-merge-split` (ModulePass)
Restructure the call graph by splitting or merging functions.

**Split**: Take a function, partition its basic blocks into N groups, extract each group into a new function. The original becomes a dispatcher that calls the pieces in sequence, passing state via struct pointer.

**Merge**: Take 2+ unrelated functions, combine into one function with an extra `i32` selector parameter. A `switch` on the selector dispatches to the original function bodies. Call sites are updated to pass the appropriate selector value.

Both transforms are applied randomly based on function size heuristics:
- Split: functions with >10 basic blocks
- Merge: pairs of functions with similar signatures (or coerce via bitcast).

#### 9. `indirect-branch` (ModulePass)
Replace direct calls and branches with function pointer table lookups.
- Build a global function pointer table (`@cobra_fptable = [N x ptr]`)
- Replace each direct `call @foo(...)` with `call @cobra_fptable[idx](...)`
- Table indices can be computed at runtime (base index + constant offset) to resist static analysis
- For internal branches within a function: replace unconditional `br` with `indirectbr` using a blockaddress table
- Applied selectively (configurable percentage of calls/branches to convert).

#### 10. `constant-unfold` (FunctionPass)
Replace compile-time constants with equivalent runtime computations.
- Integer constants: `42` → `(6 * 7)`, `0xFF` → `(0x100 - 1)`, decompose into arithmetic chains
- Use MBA expressions as building blocks for constant synthesis
- Pointer constants (null checks): `icmp eq ptr %p, null` → compare against a runtime-computed zero
- Apply to operands of instructions, not phi nodes or switch cases (those need compile-time constants).

## Architecture

```
cobra-v2/
├── CMakeLists.txt
├── include/
│   └── cobra/
│       ├── Passes.h              # Pass registration & factory functions
│       ├── PassPipeline.h        # Pipeline construction & configuration
│       └── Utils.h               # Shared utilities (RNG, opaque predicates)
├── src/
│   ├── main.cpp                  # CLI entry, module load/save, pass pipeline execution
│   ├── PassPipeline.cpp          # Pass ordering, iteration, config distribution
│   ├── passes/
│   │   ├── InsnSubstitution.cpp
│   │   ├── JunkInsertion.cpp
│   │   ├── DeadCode.cpp
│   │   ├── CFF.cpp
│   │   ├── StringEncrypt.cpp
│   │   ├── MBA.cpp
│   │   ├── BogusCF.cpp
│   │   ├── FuncMergeSplit.cpp
│   │   ├── IndirectBranch.cpp
│   │   └── ConstantUnfold.cpp
│   └── utils/
│       ├── RNG.cpp               # Seeded PRNG wrapping std::mt19937
│       └── OpaquePredicates.cpp  # Shared opaque predicate generation
└── test/
    ├── lit.cfg.py                # LLVM lit test driver configuration
    └── passes/                   # Per-pass .ll test files with FileCheck directives
        ├── insn-substitution.ll
        ├── junk-insertion.ll
        ├── dead-code.ll
        ├── cff.ll
        ├── string-encrypt.ll
        ├── mba.ll
        ├── bogus-cf.ll
        ├── func-merge-split.ll
        ├── indirect-branch.ll
        └── constant-unfold.ll
```

## Pass Interface

All passes use the LLVM new pass manager API.

**FunctionPass** pattern (most passes):
```cpp
class CFFPass : public llvm::PassInfoMixin<CFFPass> {
public:
    CFFPass(CobraConfig &config) : config(config) {}
    llvm::PreservedAnalyses run(llvm::Function &F,
                                 llvm::FunctionAnalysisManager &AM);
private:
    CobraConfig &config;
};
```

**ModulePass** pattern (string-encrypt, func-merge-split, indirect-branch):
```cpp
class StringEncryptPass : public llvm::PassInfoMixin<StringEncryptPass> {
public:
    StringEncryptPass(CobraConfig &config) : config(config) {}
    llvm::PreservedAnalyses run(llvm::Module &M,
                                 llvm::ModuleAnalysisManager &AM);
private:
    CobraConfig &config;
};
```

**Shared config struct:**
```cpp
struct CobraConfig {
    uint64_t seed;
    int iterations;
    std::vector<std::string> enabledPasses;  // empty = all
    std::vector<std::string> excludedPasses;
    bool verbose;
};
```

## Pass Ordering

Within each iteration:

```
1.  string-encrypt      (module — early, before IR gets complex)
2.  constant-unfold     (function — expand constants before other transforms)
3.  insn-substitution   (function — basic IR mutations)
4.  mba                 (function — arithmetic obfuscation)
5.  bogus-cf            (function — adds fake branches)
6.  dead-code           (function — opaque predicates + unreachable blocks)
7.  junk-insertion      (function — fill in dead weight)
8.  cff                 (function — flatten everything)
9.  func-merge-split    (module — restructure call graph)
10. indirect-branch     (module — last, replaces direct calls with indirection)
```

Rationale:
- String encryption first so decryption stubs get obfuscated by later passes
- Constant unfolding before substitution so expanded constants get further transformed
- CFF near the end so it flattens the already-obfuscated control flow
- Indirect branching last since it operates on the final call graph

The entire sequence repeats `--iterations` times.

## CLI

```
cobra-v2 [OPTIONS] <input.bc|input.ll>

Options:
  -o <output>          Output file (.bc default, .ll if --emit-ll or extension is .ll)
  --passes <list>      Comma-separated pass names, or "all" (default: all)
  --exclude <list>     Comma-separated passes to skip
  --seed <N>           RNG seed for reproducibility (default: random)
  --iterations <N>     Number of full pipeline iterations (default: 1)
  --emit-ll            Force human-readable .ll text output
  --verbose            Print per-pass statistics (functions processed, transforms applied)
```

Exit codes: 0 success, 1 error (malformed IR, I/O failure).

## Testing Strategy

### Unit tests: LLVM `lit` + `FileCheck`
Each pass gets one or more `.ll` test files:
```llvm
; RUN: cobra-v2 %s -o - --passes cff --emit-ll | FileCheck %s
; CHECK-NOT: br label %original_block
; CHECK: switch i32
define void @test_cff() {
  ; ... test IR ...
}
```

### End-to-end tests
Shell scripts that:
1. Compile a C source to `.bc`
2. Run `cobra-v2` on it
3. Compile the result to a binary
4. Run the binary and verify output matches the unobfuscated version

Test programs cover: arithmetic, control flow, string operations, function calls, recursion.

## Build

```bash
mkdir build && cd build
cmake .. -DLLVM_DIR=$(llvm-config --cmakedir)
make -j$(nproc)
```

Produces a single `cobra-v2` binary.
