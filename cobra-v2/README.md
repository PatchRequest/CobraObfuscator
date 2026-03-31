# CobraObfuscator v2

LLVM IR obfuscation toolkit. Takes LLVM bitcode or IR from any language (C, C++, Rust, Go, etc.), applies obfuscation passes, and outputs transformed IR that compiles to functionally identical but heavily obfuscated binaries.

Ships as both a **standalone CLI tool** (`cobra-v2`) and an **LLVM pass plugin** (`libcobra.dylib`) that integrates directly into compiler pipelines.

## Quick Start

### Standalone Tool

```bash
# Build
cd cobra-v2
cmake -B build -DLLVM_DIR=$(llvm-config --cmakedir)
cmake --build build

# Obfuscate C code
clang -emit-llvm -c program.c -o program.bc
./build/cobra-v2 program.bc -o program_obf.bc --passes all --seed 42
clang -O0 program_obf.bc -o program

# Obfuscate C++ code
clang++ -emit-llvm -c program.cpp -o program.bc
./build/cobra-v2 program.bc -o program_obf.bc --passes all --seed 42
clang++ -O0 program_obf.bc -o program
```

### Pass Plugin (recommended)

No intermediate files needed. One-step compilation:

```bash
# C
clang -fpass-plugin=build/libcobra.dylib -O0 program.c -o program

# C++
clang++ -fpass-plugin=build/libcobra.dylib -O0 program.cpp -o program

# With options (via environment variables)
COBRA_SEED=42 COBRA_PASSES=cff,mba,string-encrypt \
    clang -fpass-plugin=build/libcobra.dylib -O0 program.c -o program
```

> **Note:** Use `-O0` with the pass plugin. LLVM's optimizer pipeline conflicts with heavily obfuscated IR. This is correct usage -- you don't want the optimizer undoing the obfuscation.

## Obfuscation Passes

### Arithmetic & Data

| Pass | Description |
|------|-------------|
| `insn-substitution` | Replace arithmetic with semantically equivalent alternatives (`add` -> `sub(a, neg(b))`, `xor` -> `(a\|b) & ~(a&b)`) |
| `mba` | Mixed Boolean-Arithmetic -- replace operations with opaque expressions (`a+b` -> `(a^b) + 2*(a&b)`) |
| `constant-unfold` | Replace compile-time constants with runtime computations (`42` -> `6*7+0`) |
| `string-encrypt` | XOR-encrypt string constants, decrypt inline at runtime with per-string keys |

### Control Flow

| Pass | Description |
|------|-------------|
| `cff` | Control flow flattening -- replace structured control flow with a dispatcher loop. Randomly selects between three strategies: switch, if-else chain, or XOR-keyed lookup table |
| `bogus-cf` | Insert fake conditional branches using opaque predicates (always-true conditions that look dynamic) |
| `dead-code` | Insert opaque predicate guards leading to unreachable blocks filled with plausible-looking junk code |
| `junk-insertion` | Scatter volatile dead stores and loads through every basic block |

### Structural

| Pass | Description |
|------|-------------|
| `func-merge-split` | Merge pairs of functions with matching signatures into a single dispatcher function with a selector parameter |
| `indirect-branch` | Replace direct function calls with loads from a global function pointer table |
| `symbol-strip` | Rename internal functions and globals to random hex names, strip all basic block and SSA value names |
| `anti-tamper` | Insert runtime integrity checks at function entry that call `abort()` if tampered with |

## CLI Reference

```
cobra-v2 [OPTIONS] <input.bc|input.ll>

Options:
  -o <file>         Output file (default: stdout)
  --passes <list>   Comma-separated pass names or 'all' (default: all)
  --exclude <list>  Comma-separated passes to skip
  --seed <N>        RNG seed for reproducibility (default: random)
  --iterations <N>  Run the full pipeline N times (default: 1)
  --emit-ll         Output human-readable .ll instead of bitcode
  --print-stats     Print before/after module statistics
  --verbose         Print per-pass activity
  --version         Print version
```

### Examples

```bash
# Apply all passes with a fixed seed
cobra-v2 input.bc -o output.bc --passes all --seed 42

# Only CFF and string encryption
cobra-v2 input.bc -o output.bc --passes cff,string-encrypt

# Everything except junk insertion
cobra-v2 input.bc -o output.bc --passes all --exclude junk-insertion

# Double obfuscation (apply all passes twice)
cobra-v2 input.bc -o output.bc --passes all --iterations 2

# Inspect the obfuscated IR
cobra-v2 input.bc -o output.ll --emit-ll --passes cff --seed 42

# See statistics
cobra-v2 input.bc -o output.bc --passes all --print-stats
```

## Pass Plugin Reference

The plugin is configured via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `COBRA_SEED` | random | RNG seed |
| `COBRA_PASSES` | `all` | Comma-separated pass list |
| `COBRA_EXCLUDE` | (none) | Passes to skip |
| `COBRA_ITERATIONS` | `1` | Pipeline iterations |
| `COBRA_VERBOSE` | `0` | Set to `1` for verbose output |

```bash
# Selective passes
COBRA_PASSES=cff,mba,string-encrypt clang -fpass-plugin=libcobra.dylib -O0 foo.c -o foo

# Fixed seed for reproducible builds
COBRA_SEED=12345 clang -fpass-plugin=libcobra.dylib -O0 foo.c -o foo

# Exclude slow passes
COBRA_EXCLUDE=junk-insertion,dead-code clang -fpass-plugin=libcobra.dylib -O0 foo.c -o foo

# See what's happening
COBRA_VERBOSE=1 clang -fpass-plugin=libcobra.dylib -O0 foo.c -o foo 2>&1
```

## Pass Ordering

Passes run in this order within each iteration:

```
1. string-encrypt        (module -- encrypt before other passes obscure the IR)
2. constant-unfold       (function -- expand constants before further transforms)
3. insn-substitution     (function)
4. mba                   (function)
5. bogus-cf              (function)
6. dead-code             (function)
7. junk-insertion        (function)
8. cff                   (function -- flatten the already-obfuscated control flow)
9. anti-tamper           (function -- integrity checks on the final obfuscated code)
10. func-merge-split     (module)
11. indirect-branch      (module)
12. symbol-strip         (module -- rename everything last)
```

With `--iterations 2`, the entire sequence runs twice for layered obfuscation.

## Building

### Requirements

- LLVM 17+ (tested with LLVM 20)
- CMake 3.20+
- C++17 compiler

### macOS (Homebrew)

```bash
brew install llvm cmake
cd cobra-v2
cmake -B build -DLLVM_DIR=$(brew --prefix llvm)/lib/cmake/llvm
cmake --build build
```

Produces:
- `build/cobra-v2` -- standalone CLI tool
- `build/libcobra.dylib` -- LLVM pass plugin

### Linux

```bash
apt install llvm-20-dev cmake
cd cobra-v2
cmake -B build -DLLVM_DIR=/usr/lib/llvm-20/lib/cmake/llvm
cmake --build build
```

Produces `build/cobra-v2` and `build/libcobra.so`.

## Language Support

| Language | Standalone Tool | Pass Plugin |
|----------|----------------|-------------|
| C | Full support | Full support (`clang -fpass-plugin=...`) |
| C++ | Full support | Full support (`clang++ -fpass-plugin=...`) |
| Rust | Emit BC with `rustc --emit=llvm-bc`, obfuscate, link with `clang` | Requires building against rustc's LLVM version |
| Any LLVM language | Full support via BC/IR | Depends on compiler's plugin support |

### Rust Workflow (standalone tool)

```bash
# Single file
rustc --emit=llvm-bc main.rs -o main.bc
cobra-v2 main.bc -o main_obf.bc --passes all --seed 42
clang -O0 main_obf.bc -o main    # works for #![no_std] programs

# For programs using std, you need to link against the Rust runtime
# (use the standalone tool on individual crate .bc files within a cargo build)
```

## Testing

```bash
# Run the full E2E test suite (350 tests, ~2 hours)
./test/e2e/run_e2e.sh

# Quick smoke test
clang -emit-llvm -c test/e2e/arith.c -o /tmp/arith.bc
./build/cobra-v2 /tmp/arith.bc -o /tmp/arith_obf.bc --passes all --seed 42
clang -O0 /tmp/arith_obf.bc -o /tmp/arith_obf -lm
/tmp/arith_obf   # should print: add=13 sub=7 xor=9
```

## Architecture

```
cobra-v2/
├── include/cobra/
│   ├── CobraConfig.h       # Configuration struct
│   ├── Passes.h            # All pass class declarations
│   ├── PassPipeline.h      # Pipeline runner interface
│   ├── RNG.h               # Seeded PRNG
│   ├── OpaquePredicates.h  # Shared opaque predicate builder
│   └── Stats.h             # Module statistics
├── src/
│   ├── main.cpp            # Standalone CLI entry point
│   ├── Plugin.cpp          # LLVM pass plugin entry point
│   ├── PassPipeline.cpp    # Pipeline construction and execution
│   ├── Stats.cpp           # Statistics implementation
│   ├── passes/             # One file per obfuscation pass
│   │   ├── InsnSubstitution.cpp
│   │   ├── MBA.cpp
│   │   ├── ConstantUnfold.cpp
│   │   ├── DeadCode.cpp
│   │   ├── BogusCF.cpp
│   │   ├── JunkInsertion.cpp
│   │   ├── CFF.cpp
│   │   ├── AntiTamper.cpp
│   │   ├── StringEncrypt.cpp
│   │   ├── FuncMergeSplit.cpp
│   │   ├── IndirectBranch.cpp
│   │   └── SymbolStrip.cpp
│   └── utils/
│       └── OpaquePredicates.cpp
└── test/
    ├── passes/             # Per-pass .ll tests
    └── e2e/                # End-to-end correctness tests (C, C++)
```
