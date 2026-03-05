#!/bin/bash
# ============================================================================
# Cobra Obfuscator — Full Test Matrix
# ============================================================================
# Compilers: GCC (MinGW), Clang (MinGW target), MSVC (cl.exe)
# Opt levels: -O0, -O1, -O2, -Os, -O3 (GCC/Clang); /Od, /O1, /O2 (MSVC)
# Test programs: minimal, medium, loops, recursion, switch_heavy, selfval,
#                bitops, structs, func_ptrs
# Rust programs: rust_crypto, rust_structs (debug + release)
# Go programs:   go_algorithms, go_crypto
# Pass combos: all, cff-only, junk-only, dead-only, insn-only, cff+junk, etc.
# Seeds: 1..10
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COBRA="$PROJECT_DIR/target/release/cobra-obfuscator.exe"
WORK_DIR="/c/tmp/cobra_test_matrix"
RESULTS_FILE="$WORK_DIR/results.csv"
SUMMARY_FILE="$WORK_DIR/summary.txt"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# ---- Compiler detection ----
GCC="$(which gcc 2>/dev/null || true)"
CLANG="$(which clang 2>/dev/null || true)"
RUSTC="$(which rustc 2>/dev/null || true)"
CARGO="$(which cargo 2>/dev/null || true)"
GO="$(which go 2>/dev/null || true)"

# MSVC: detect via vcvarsall.bat and create a compile wrapper
MSVC=""
VCVARSALL=""
for vs_path in \
    "/c/Program Files/Microsoft Visual Studio/2022/Community" \
    "/c/Program Files/Microsoft Visual Studio/2022/Professional" \
    "/c/Program Files/Microsoft Visual Studio/2022/Enterprise" \
    "/c/Program Files (x86)/Microsoft Visual Studio/2019/Community" \
    "/c/Program Files (x86)/Microsoft Visual Studio/2019/Professional"; do
    if [ -f "$vs_path/VC/Auxiliary/Build/vcvarsall.bat" ]; then
        # Convert to Windows path
        VCVARSALL="$(cd "$vs_path/VC/Auxiliary/Build" && pwd -W)/vcvarsall.bat"
        break
    fi
done

if [ -n "$VCVARSALL" ]; then
    # Create a bat wrapper for MSVC compilation
    MSVC_BAT="$WORK_DIR/msvc_compile.bat"
    mkdir -p "$WORK_DIR"
    cat > "$MSVC_BAT" << BATEOF
@echo off
call "$VCVARSALL" x64 >nul 2>&1
cl.exe /nologo %*
BATEOF
    MSVC="msvc"
fi

echo "============================================"
echo "  Cobra Obfuscator — Test Matrix Runner"
echo "============================================"
echo ""
echo "Compilers:"
[ -n "$GCC" ]   && echo "  GCC:   $GCC"   || echo "  GCC:   NOT FOUND"
[ -n "$CLANG" ] && echo "  Clang: $CLANG" || echo "  Clang: NOT FOUND"
[ -n "$MSVC" ]  && echo "  MSVC:  $VCVARSALL" || echo "  MSVC:  NOT FOUND"
[ -n "$RUSTC" ] && echo "  Rust:  $RUSTC ($(rustc --version 2>/dev/null || echo '?'))" || echo "  Rust:  NOT FOUND"
[ -n "$GO" ]    && echo "  Go:    $GO ($(go version 2>/dev/null || echo '?'))"          || echo "  Go:    NOT FOUND"
echo ""

# ---- Setup ----
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR/bins" "$WORK_DIR/obf"

# Re-create MSVC bat wrapper after rm -rf
if [ -n "$VCVARSALL" ]; then
    MSVC_BAT="$WORK_DIR/msvc_compile.bat"
    cat > "$MSVC_BAT" << BATEOF
@echo off
call "$VCVARSALL" x64 >nul 2>&1
cl.exe /nologo %*
BATEOF
fi

ALL_C_TESTS="minimal medium loops recursion switch_heavy selfval bitops structs func_ptrs"
ALL_RUST_TESTS="rust_crypto rust_structs"
ALL_GO_TESTS="go_algorithms go_crypto"

# ---- Pass combinations (label:disable_flags) ----
PASS_COMBOS=(
    "all:"
    "cff-only:junk-insertion,dead-code,insn-substitution"
    "junk-only:control-flow-flatten,dead-code,insn-substitution"
    "dead-only:control-flow-flatten,junk-insertion,insn-substitution"
    "insn-only:control-flow-flatten,junk-insertion,dead-code"
    "cff+junk:dead-code,insn-substitution"
    "cff+dead:junk-insertion,insn-substitution"
    "cff+insn:junk-insertion,dead-code"
    "junk+dead:control-flow-flatten,insn-substitution"
    "cff+junk+dead:insn-substitution"
    "cff+junk+insn:dead-code"
)

SEEDS="1 2 3 4 5 6 7 8 9 10"

echo "compiler,opt_level,test_program,pass_combo,seed,result,exit_code" > "$RESULTS_FILE"

# ---- Compile helper ----
compile_test() {
    local compiler_name="$1" src="$2" opt="$3" out="$4"
    case "$compiler_name" in
        gcc)
            "$GCC" $opt -o "$out" "$src" -lkernel32 2>/dev/null
            ;;
        clang)
            "$CLANG" --target=x86_64-w64-mingw32 $opt -o "$out" "$src" -lkernel32 2>/dev/null
            ;;
        msvc)
            # Convert paths to Windows format for cmd.exe
            local win_out win_src
            win_out="$(cygpath -w "$out" 2>/dev/null || echo "$out")"
            win_src="$(cygpath -w "$src" 2>/dev/null || echo "$src")"
            local win_bat
            win_bat="$(cygpath -w "$MSVC_BAT" 2>/dev/null || echo "$MSVC_BAT")"
            cmd.exe //C "$win_bat /Fe:$win_out $opt $win_src" >/dev/null 2>&1
            # cl.exe leaves .obj files in current dir — clean up
            local base
            base="$(basename "$src" .c)"
            rm -f "${base}.obj" 2>/dev/null
            ;;
    esac
}

# ---- Run one test ----
run_test() {
    local compiler_name="$1" opt="$2" test_name="$3" pass_label="$4" disable_flags="$5" seed="$6" input_exe="$7"
    local safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
    local out_name="${compiler_name}_${safe_opt}_${test_name}_${pass_label}_s${seed}"
    local out_exe="$WORK_DIR/obf/${out_name}.exe"

    TOTAL=$((TOTAL + 1))

    local cmd="$COBRA -i $input_exe -o $out_exe --seed $seed"
    [ -n "$disable_flags" ] && cmd="$cmd --disable $disable_flags"

    if ! eval "$cmd" >/dev/null 2>&1; then
        echo -e "  ${RED}FAIL${NC} [obf-err] $compiler_name $opt $test_name $pass_label s=$seed"
        echo "$compiler_name,$opt,$test_name,$pass_label,$seed,OBF_ERROR,-1" >> "$RESULTS_FILE"
        FAILED=$((FAILED + 1))
        return
    fi

    local exit_code=0
    timeout 30 "$out_exe" >/dev/null 2>&1 || exit_code=$?

    if [ "$exit_code" -eq 0 ]; then
        echo "$compiler_name,$opt,$test_name,$pass_label,$seed,PASS,0" >> "$RESULTS_FILE"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}FAIL${NC} [exit=$exit_code] $compiler_name $opt $test_name $pass_label s=$seed"
        echo "$compiler_name,$opt,$test_name,$pass_label,$seed,FAIL,$exit_code" >> "$RESULTS_FILE"
        FAILED=$((FAILED + 1))
    fi

    rm -f "$out_exe"
}

# ---- Run all combos for a single binary ----
run_all_combos() {
    local compiler_name="$1" opt="$2" test_name="$3" bin="$4"

    for combo in "${PASS_COMBOS[@]}"; do
        label="${combo%%:*}"
        disable="${combo#*:}"
        for seed in $SEEDS; do
            run_test "$compiler_name" "$opt" "$test_name" "$label" "$disable" "$seed" "$bin"
        done
    done
}

# ---- Skip all combos for a test ----
skip_all_combos() {
    local compiler_name="$1" opt="$2" test_name="$3" reason="$4" exit_code="$5"

    for combo in "${PASS_COMBOS[@]}"; do
        label="${combo%%:*}"
        for seed in $SEEDS; do
            TOTAL=$((TOTAL + 1)); SKIPPED=$((SKIPPED + 1))
            echo "$compiler_name,$opt,$test_name,$label,$seed,${reason},${exit_code}" >> "$RESULTS_FILE"
        done
    done
}

# ============================================================================
# Part 1: C test programs (GCC, Clang, MSVC × optimization levels)
# ============================================================================
declare -A COMPILER_OPTS
COMPILER_OPTS[gcc]="-O0 -O1 -O2 -Os -O3"
COMPILER_OPTS[clang]="-O0 -O1 -O2 -Os -O3"
COMPILER_OPTS[msvc]="/Od /O1 /O2"

AVAILABLE_COMPILERS=""
[ -n "$GCC" ]   && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS gcc"
[ -n "$CLANG" ] && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS clang"
[ -n "$MSVC" ]  && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS msvc"

for compiler in $AVAILABLE_COMPILERS; do
    echo ""
    echo -e "${CYAN}===== Compiler: $compiler =====${NC}"

    for opt in ${COMPILER_OPTS[$compiler]}; do
        echo -e "  ${YELLOW}--- $opt ---${NC}"

        for test_name in $ALL_C_TESTS; do
            src="$SCRIPT_DIR/${test_name}.c"
            safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
            bin="$WORK_DIR/bins/${compiler}_${safe_opt}_${test_name}.exe"

            if ! compile_test "$compiler" "$src" "$opt" "$bin" 2>/dev/null; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name — compile failed"
                skip_all_combos "$compiler" "$opt" "$test_name" "COMPILE_FAIL" "-1"
                continue
            fi

            local_exit=0
            timeout 30 "$bin" >/dev/null 2>&1 || local_exit=$?
            if [ "$local_exit" -ne 0 ]; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name — original fails (exit=$local_exit)"
                skip_all_combos "$compiler" "$opt" "$test_name" "ORIG_FAIL" "$local_exit"
                continue
            fi

            run_all_combos "$compiler" "$opt" "$test_name" "$bin"
        done
    done
done

# ============================================================================
# Part 2: Rust test programs (debug + release builds via cargo)
# ============================================================================
if [ -n "$CARGO" ]; then
    echo ""
    echo -e "${CYAN}===== Compiler: rust (cargo) =====${NC}"

    for test_name in $ALL_RUST_TESTS; do
        test_dir="$SCRIPT_DIR/$test_name"
        if [ ! -f "$test_dir/Cargo.toml" ]; then
            echo -e "  ${YELLOW}SKIP${NC} $test_name — no Cargo.toml"
            continue
        fi

        for profile in debug release; do
            echo -e "  ${YELLOW}--- $test_name ($profile) ---${NC}"

            if [ "$profile" = "release" ]; then
                cargo_flag="--release"
                target_subdir="release"
            else
                cargo_flag=""
                target_subdir="debug"
            fi

            bin="$WORK_DIR/bins/rust_${profile}_${test_name}.exe"

            if ! (cd "$test_dir" && cargo build $cargo_flag 2>/dev/null); then
                echo -e "    ${YELLOW}SKIP${NC} $test_name ($profile) — cargo build failed"
                skip_all_combos "rust" "$profile" "$test_name" "COMPILE_FAIL" "-1"
                continue
            fi

            # Find the built binary
            built_exe="$test_dir/target/$target_subdir/${test_name}.exe"
            if [ ! -f "$built_exe" ]; then
                # Try with hyphens replaced by underscores
                alt_name="${test_name//-/_}"
                built_exe="$test_dir/target/$target_subdir/${alt_name}.exe"
            fi
            if [ ! -f "$built_exe" ]; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name ($profile) — binary not found"
                skip_all_combos "rust" "$profile" "$test_name" "COMPILE_FAIL" "-1"
                continue
            fi

            cp "$built_exe" "$bin"

            local_exit=0
            timeout 30 "$bin" >/dev/null 2>&1 || local_exit=$?
            if [ "$local_exit" -ne 0 ]; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name ($profile) — original fails (exit=$local_exit)"
                skip_all_combos "rust" "$profile" "$test_name" "ORIG_FAIL" "$local_exit"
                continue
            fi

            run_all_combos "rust" "$profile" "$test_name" "$bin"
        done
    done
else
    echo ""
    echo -e "${YELLOW}Rust (cargo) not found — skipping Rust tests${NC}"
fi

# ============================================================================
# Part 3: Go test programs
# ============================================================================
# Go binaries are auto-detected and obfuscated in-place (preserving .gopclntab).
# Only the "all" pass combo is tested since in-place mode has size constraints.
if [ -n "$GO" ]; then
    echo ""
    echo -e "${CYAN}===== Compiler: go (in-place mode) =====${NC}"

    for test_name in $ALL_GO_TESTS; do
        test_dir="$SCRIPT_DIR/$test_name"
        if [ ! -f "$test_dir/main.go" ]; then
            echo -e "  ${YELLOW}SKIP${NC} $test_name — no main.go"
            continue
        fi

        echo -e "  ${YELLOW}--- $test_name ---${NC}"

        bin="$WORK_DIR/bins/go_${test_name}.exe"

        if ! (cd "$test_dir" && GOOS=windows GOARCH=amd64 go build -o "$bin" . 2>/dev/null); then
            echo -e "    ${YELLOW}SKIP${NC} $test_name — go build failed"
            skip_all_combos "go" "default" "$test_name" "COMPILE_FAIL" "-1"
            continue
        fi

        local_exit=0
        timeout 30 "$bin" >/dev/null 2>&1 || local_exit=$?
        if [ "$local_exit" -ne 0 ]; then
            echo -e "    ${YELLOW}SKIP${NC} $test_name — original fails (exit=$local_exit)"
            skip_all_combos "go" "default" "$test_name" "ORIG_FAIL" "$local_exit"
            continue
        fi

        run_all_combos "go" "default" "$test_name" "$bin"
    done
else
    echo ""
    echo -e "${YELLOW}Go not found — skipping Go tests${NC}"
fi

# ---- Summary ----
echo ""
echo "============================================"
echo "  TEST MATRIX RESULTS"
echo "============================================"
echo ""
echo -e "  Total:   $TOTAL"
echo -e "  ${GREEN}Passed:  $PASSED${NC}"
echo -e "  ${RED}Failed:  $FAILED${NC}"
echo -e "  ${YELLOW}Skipped: $SKIPPED${NC}"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "Failures:"
    grep ",FAIL,\|OBF_ERROR" "$RESULTS_FILE" | grep -v "COMPILE_FAIL\|ORIG_FAIL" | head -50
fi

# ---- Summary file ----
ALL_TESTS="$ALL_C_TESTS $ALL_RUST_TESTS $ALL_GO_TESTS"
ALL_OPT_LEVELS="-O0 -O1 -O2 -Os -O3 /Od /O1 /O2 debug release default"
ALL_COMPILERS="$AVAILABLE_COMPILERS"
[ -n "$CARGO" ] && ALL_COMPILERS="$ALL_COMPILERS rust"
[ -n "$GO" ]    && ALL_COMPILERS="$ALL_COMPILERS go"

{
    echo "============================================"
    echo "  Cobra Obfuscator Test Matrix Summary"
    echo "  $(date)"
    echo "============================================"
    echo ""
    echo "Total: $TOTAL  Passed: $PASSED  Failed: $FAILED  Skipped: $SKIPPED"
    echo ""

    echo "--- By Compiler ---"
    for compiler in $ALL_COMPILERS; do
        p=$(grep "^$compiler," "$RESULTS_FILE" | grep -c ",PASS," || true)
        f=$(grep "^$compiler," "$RESULTS_FILE" | grep ",FAIL,\|OBF_ERROR" | grep -vc "COMPILE_FAIL\|ORIG_FAIL" || true)
        s=$(grep "^$compiler," "$RESULTS_FILE" | grep -c "COMPILE_FAIL\|ORIG_FAIL\|SKIP" || true)
        echo "  $compiler: $p pass, $f fail, $s skip"
    done
    echo ""

    echo "--- By Optimization Level ---"
    for opt in $ALL_OPT_LEVELS; do
        p=$(grep ",$opt," "$RESULTS_FILE" | grep -c ",PASS," || true)
        f=$(grep ",$opt," "$RESULTS_FILE" | grep ",FAIL,\|OBF_ERROR" | grep -vc "COMPILE_FAIL\|ORIG_FAIL" || true)
        s=$(grep ",$opt," "$RESULTS_FILE" | grep -c "COMPILE_FAIL\|ORIG_FAIL" || true)
        [ $((p + f + s)) -gt 0 ] && echo "  $opt: $p pass, $f fail, $s skip"
    done
    echo ""

    echo "--- By Test Program ---"
    for test_name in $ALL_TESTS; do
        p=$(grep ",$test_name," "$RESULTS_FILE" | grep -c ",PASS," || true)
        f=$(grep ",$test_name," "$RESULTS_FILE" | grep ",FAIL,\|OBF_ERROR" | grep -vc "COMPILE_FAIL\|ORIG_FAIL" || true)
        s=$(grep ",$test_name," "$RESULTS_FILE" | grep -c "COMPILE_FAIL\|ORIG_FAIL" || true)
        echo "  $test_name: $p pass, $f fail, $s skip"
    done
    echo ""

    echo "--- By Pass Combination ---"
    for combo in "${PASS_COMBOS[@]}"; do
        label="${combo%%:*}"
        p=$(grep ",$label," "$RESULTS_FILE" | grep -c ",PASS," || true)
        f=$(grep ",$label," "$RESULTS_FILE" | grep ",FAIL,\|OBF_ERROR" | grep -vc "COMPILE_FAIL\|ORIG_FAIL" || true)
        echo "  $label: $p pass, $f fail"
    done
    echo ""

    if [ "$FAILED" -gt 0 ]; then
        echo "--- All Failures ---"
        grep ",FAIL,\|OBF_ERROR" "$RESULTS_FILE" | grep -v "COMPILE_FAIL\|ORIG_FAIL"
    fi
} > "$SUMMARY_FILE"

echo ""
echo "Results: $RESULTS_FILE"
echo "Summary: $SUMMARY_FILE"

if [ "$FAILED" -eq 0 ] && [ "$PASSED" -gt 0 ]; then
    echo -e "\n${GREEN}ALL TESTS PASSED!${NC}"
    exit 0
else
    echo -e "\n${RED}SOME TESTS FAILED${NC}"
    exit 1
fi
