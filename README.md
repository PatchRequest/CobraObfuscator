```
   ██████╗ ██████╗ ██████╗ ██████╗  █████╗
  ██╔════╝██╔═══██╗██╔══██╗██╔══██╗██╔══██╗
  ██║     ██║   ██║██████╔╝██████╔╝███████║
  ██║     ██║   ██║██╔══██╗██╔══██╗██╔══██║
  ╚██████╗╚██████╔╝██████╔╝██║  ██║██║  ██║
   ╚═════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

# Cobra Obfuscator

**Post-compilation x86-64 binary obfuscator for Windows PE executables and DLLs.**

Cobra operates directly on compiled `.exe` and `.dll` binaries — no source code, no recompilation, no compiler plugins. Hand it a PE binary and it produces an obfuscated copy with transformed function bodies scattered throughout the existing `.text` section.

Supports binaries compiled with **GCC (MinGW)**, **Clang**, **MSVC**, **Rust**, and **Go**.

## Features

### Control Flow Flattening (CFF)
Destroys the natural control flow graph by replacing it with a dispatcher-based state machine. Each basic block becomes a case in a central switch, with transitions driven by a state register (`R15`). Static analysis tools see a single flat loop instead of the original branching structure.

### Junk Code Insertion
Injects semantically-inert instruction sequences between real instructions — NOPs, self-canceling XORs, balanced ADD/SUB pairs. Inflates function bodies and breaks signature-based pattern matching without affecting program behavior.

### Dead Code Insertion
Adds unreachable code blocks guarded by opaque predicates (`cmp rsp, 0; je ...` — RSP is never zero). These dead paths contain realistic-looking instruction sequences that waste analyst time and confuse decompilers.

### Instruction Substitution
Replaces instructions with semantically equivalent alternatives. Breaks byte-level signatures while preserving exact program semantics.

### Intelligent CRT Detection
Automatically identifies CRT/runtime functions via inverted call graph analysis — BFS from the entry point marks runtime code, everything else is user code. Aggressive passes (CFF, dead-code) are only applied to confirmed user functions, preventing breakage of statically-linked library code.

### Obfuscation Statistics
Reports detailed metrics after each run: .text coverage, function counts, expansion ratio, and pass breakdown.

### Stealth Code Placement
Three placement modes keep obfuscated code hidden within existing PE structure:

- **Scatter** (GCC/Clang/Rust) — obfuscated functions are placed into code caves within `.text`: original function bodies (after trampolining) and inter-function alignment padding. ~84% of functions fit inside `.text` caves; overflow extends the last section.
- **Extension** (MSVC) — obfuscated code appended to the last section. Avoids disturbing MSVC-specific structures (ICF, security cookies).
- **In-place** (Go) — obfuscated code written back at the original function address, preserving PC-to-metadata mappings (`.gopclntab`). Functions that grow too large are skipped.

No new sections are added. No suspicious section names.

### Seed-Based Reproducibility
All transforms use a seeded PRNG. Same seed, same output — useful for debugging, CI, and deterministic builds.

## How It Works

1. **Parse** the PE binary — reads section headers, `.pdata` exception table, relocations
2. **Discover** functions via `.pdata` RUNTIME_FUNCTION entries
3. **Classify** — inverted CRT detection identifies runtime vs user functions via call graph BFS from entry point
4. **Lift** each function into an IR (basic blocks + CFG)
5. **Transform** — runs the pass pipeline with per-function pass selection:
   - All user functions: `insn_substitution → junk_insertion`
   - Main-reachable functions: additionally `dead_code → control_flow_flatten`
6. **Encode** transformed functions into machine code
7. **Scatter** obfuscated code into caves within `.text` — original function bodies, inter-function padding, and overflow appended to the last section
8. **Patch** original functions with `jmp` trampolines redirecting to their new locations

## Usage

```bash
# Basic usage — works on .exe and .dll
cobra-obfuscator -i target.exe -o target_obf.exe
cobra-obfuscator -i library.dll -o library_obf.dll

# With a fixed seed for reproducibility
cobra-obfuscator -i target.exe -o target_obf.exe --seed 42

# Disable specific passes
cobra-obfuscator -i target.exe -o target_obf.exe --disable junk-insertion,dead-code

# Multiple iterations (stack transforms)
cobra-obfuscator -i target.exe -o target_obf.exe --iterations 2

# Adjust junk density (0.0–1.0)
cobra-obfuscator -i target.exe -o target_obf.exe --junk-density 0.5
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `-i, --input` | Input PE binary (`.exe` or `.dll`) | required |
| `-o, --output` | Output file path | required |
| `--seed` | RNG seed for deterministic output | random |
| `--iterations` | Number of pass iterations | `1` |
| `--disable` | Comma-separated passes to skip | none |
| `--junk-density` | Junk insertion probability (0.0–1.0) | `0.3` |
| `--format` | Force input format: `auto`, `coff`, `pe` | `auto` |

### Pass Names (for `--disable`)

- `control-flow-flatten`
- `junk-insertion`
- `dead-code`
- `insn-substitution`

## Building

```bash
cargo build --release
```

The binary lands in `target/release/cobra-obfuscator.exe`.

**Requirements:** Rust toolchain (stable). No external dependencies beyond crates.

## Test Suite

The test matrix covers multiple compilers, optimization levels, pass combinations, and seeds:

- **Compilers** — GCC (MinGW), Clang (MinGW target), MSVC (cl.exe), Rust (debug + release), Go
- **Optimization levels** — `-O0` through `-O3`, `-Os` (GCC/Clang), `/Od`, `/O1`, `/O2` (MSVC)
- **9 C test programs** — minimal, medium, loops, recursion, switch_heavy, selfval, bitops, structs, func_ptrs
- **2 Rust test programs** — rust_crypto, rust_structs (debug + release builds)
- **2 Go test programs** — go_algorithms, go_crypto
- **DLL tests** — C DLL and Rust cdylib with loader programs that validate exported functions
- **11 pass combinations** — all passes, each pass solo, pairwise combos, triple combos
- **10 seeds** per configuration

Test programs exercise: deep nesting, mutual recursion, Ackermann function, binary exponentiation, bubble sort, state machines, large/sparse/nested switch statements, heap allocation, function pointers, bitwise operations, structs, hashing, sorting, primality testing, and cryptographic operations.

```bash
# Run the full matrix (auto-detects available compilers)
cd tests && bash run_matrix.sh
```

### Coverage

| Target | .text Coverage | Mode | Status |
|--------|---------------|------|--------|
| C (GCC, 9 programs) | 58–63% | Scatter | All pass |
| C (Clang, 9 programs) | 58–63% | Scatter | All pass |
| C (MSVC, 9 programs) | 55–60% | Extension | All pass |
| Rust (debug + release) | 76–84% | Scatter | All pass |
| Go (2 programs) | 40–50% | In-place | All pass |
| C DLL (GCC) | ~68% | Scatter | All pass |
| Rust DLL (cdylib) | ~80% | Scatter | All pass |

**13,530 tests** across all compilers, optimization levels, pass combinations, and seeds — all passing.

## Architecture

```
src/
├── main.rs              CLI entry point
├── lib.rs               Library API
├── config.rs            ObfuscatorConfig
├── pipeline.rs          Orchestrates PE/COFF obfuscation + statistics
├── passes/
│   ├── pass_trait.rs    ObfuscationPass trait + PassContext
│   ├── control_flow_flatten.rs
│   ├── junk_insertion.rs
│   ├── dead_code.rs
│   └── insn_substitution.rs
├── ir/
│   ├── instruction.rs   IrInsn (wraps iced-x86)
│   ├── basic_block.rs   BasicBlock + successors
│   ├── function.rs      Function IR container
│   ├── cfg.rs           CFG construction
│   └── relocation.rs    Relocation tracking
├── pe/
│   ├── reader.rs        PE parsing, function discovery, CRT detection
│   ├── writer.rs        Scatter/extension/in-place writers + trampolines
│   ├── pdata.rs         Exception table parsing
│   ├── reloc.rs         PE relocation handling
│   └── types.rs         PE data structures
├── coff/
│   ├── reader.rs        COFF object file parsing
│   ├── writer.rs        COFF output
│   └── types.rs         COFF data structures
└── encode/
    ├── assembler.rs     IR → machine code (BlockEncoder)
    └── reloc_fixup.rs   Post-encode relocation validation
```

## Limitations

- **x86-64 only** — no 32-bit support
- **Windows PE only** — no ELF/Mach-O (yet)
- Functions with jump tables (indirect branches) are skipped to preserve correctness
- CRT/runtime functions are intentionally excluded from transformation
- Go binaries use in-place mode (code stays at original addresses) to preserve `.gopclntab` metadata

## License

MIT
