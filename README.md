```
   ██████╗ ██████╗ ██████╗ ██████╗  █████╗
  ██╔════╝██╔═══██╗██╔══██╗██╔══██╗██╔══██╗
  ██║     ██║   ██║██████╔╝██████╔╝███████║
  ██║     ██║   ██║██╔══██╗██╔══██╗██╔══██║
  ╚██████╗╚██████╔╝██████╔╝██║  ██║██║  ██║
   ╚═════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

# Cobra Obfuscator

**Post-compilation x86-64 binary obfuscator for Windows PE executables.**

Cobra operates directly on compiled `.exe` and `.dll` binaries — no source code, no recompilation, no compiler plugins. Hand it a PE binary and it produces an obfuscated copy with a new `.cobra` code section containing transformed function bodies.

## Features

### Control Flow Flattening (CFF)
Destroys the natural control flow graph by replacing it with a dispatcher-based state machine. Each basic block becomes a case in a central switch, with transitions driven by a state register (`R15`). Static analysis tools see a single flat loop instead of the original branching structure.

### Junk Code Insertion
Injects semantically-inert instruction sequences between real instructions — NOPs, self-canceling XORs, balanced ADD/SUB pairs. Inflates function bodies and breaks signature-based pattern matching without affecting program behavior.

### Dead Code Insertion
Adds unreachable code blocks guarded by opaque predicates (`cmp rsp, 0; je ...` — RSP is never zero). These dead paths contain realistic-looking instruction sequences that waste analyst time and confuse decompilers.

### Instruction Substitution
Replaces instructions with semantically equivalent alternatives. Breaks byte-level signatures while preserving exact program semantics.

### Seed-Based Reproducibility
All transforms use a seeded PRNG. Same seed, same output — useful for debugging, CI, and deterministic builds.

## How It Works

1. **Parse** the PE binary — reads section headers, `.pdata` exception table, relocations
2. **Discover** functions via BFS from the entry point through the call graph
3. **Filter** — skips CRT/runtime functions, tiny stubs, and functions with indirect branches (jump tables)
4. **Lift** each function into an IR (basic blocks + CFG)
5. **Transform** — runs the pass pipeline: `insn_substitution → junk_insertion → dead_code → control_flow_flatten`
6. **Encode** transformed functions into machine code
7. **Emit** a new `.cobra` section containing all obfuscated function bodies
8. **Patch** original functions with `jmp` trampolines redirecting into `.cobra`

## Usage

```bash
# Basic usage
cobra-obfuscator -i target.exe -o target_obf.exe

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

The test matrix covers **8,580 configurations** across:

- **3 compilers** — GCC (MinGW), Clang (MinGW target), MSVC (cl.exe)
- **5 optimization levels** — `-O0`, `-O1`, `-O2`, `-Os`, `-O3` (GCC/Clang) + `/Od`, `/O1`, `/O2` (MSVC)
- **6 test programs** — minimal, medium, loops, recursion, switch_heavy, selfval
- **11 pass combinations** — all passes, each pass solo, pairwise combos, triple combos
- **10 seeds** per configuration

Test programs exercise: deep nesting, mutual recursion, Ackermann function, binary exponentiation, bubble sort, state machines, large/sparse/nested switch statements, heap allocation, TLS callbacks, threading, VEH, and WinAPI calls.

```bash
# Run the full matrix (auto-detects available compilers)
cd tests && bash run_matrix.sh
```

## Architecture

```
src/
├── main.rs              CLI entry point
├── config.rs            ObfuscatorConfig
├── pipeline.rs          Orchestrates PE/COFF obfuscation
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
│   ├── reader.rs        PE parsing, function discovery
│   ├── writer.rs        .cobra section creation + trampolines
│   ├── pdata.rs         Exception table parsing
│   └── reloc.rs         PE relocation handling
└── encode/
    ├── assembler.rs     IR → machine code
    └── reloc_fixup.rs   Post-encode relocation validation
```

## Limitations

- **x86-64 only** — no 32-bit support
- **Windows PE only** — no ELF/Mach-O (yet)
- Functions with jump tables (indirect branches) are skipped to preserve correctness
- CRT/runtime functions are intentionally excluded from transformation

## License

MIT
