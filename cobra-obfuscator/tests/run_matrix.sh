#!/bin/bash
# ============================================================================
# Cobra Obfuscator — Full Test Matrix
# ============================================================================
# Compilers: GCC (MinGW), Clang (MinGW target)
# Opt levels: -O0, -O1, -O2, -Os, -O3
# Test programs: minimal, medium, loops, recursion, switch_heavy, selfval
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

echo "============================================"
echo "  Cobra Obfuscator — Test Matrix Runner"
echo "============================================"
echo ""
echo "Compilers:"
[ -n "$GCC" ]   && echo "  GCC:   $GCC"   || echo "  GCC:   NOT FOUND"
[ -n "$CLANG" ] && echo "  Clang: $CLANG" || echo "  Clang: NOT FOUND"
echo ""

# ---- Setup ----
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR/bins" "$WORK_DIR/obf"

ALL_TESTS="minimal medium loops recursion switch_heavy selfval"

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
    esac
}

# ---- Run one test ----
run_test() {
    local compiler_name="$1" opt="$2" test_name="$3" pass_label="$4" disable_flags="$5" seed="$6" input_exe="$7"
    local out_name="${compiler_name}_${opt//-/}_${test_name}_${pass_label}_s${seed}"
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

# ---- Main loop ----
declare -A COMPILER_OPTS
COMPILER_OPTS[gcc]="-O0 -O1 -O2 -Os -O3"
COMPILER_OPTS[clang]="-O0 -O1 -O2 -Os -O3"

AVAILABLE_COMPILERS=""
[ -n "$GCC" ]   && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS gcc"
[ -n "$CLANG" ] && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS clang"

for compiler in $AVAILABLE_COMPILERS; do
    echo ""
    echo -e "${CYAN}===== Compiler: $compiler =====${NC}"

    for opt in ${COMPILER_OPTS[$compiler]}; do
        echo -e "  ${YELLOW}--- $opt ---${NC}"

        for test_name in $ALL_TESTS; do
            src="$SCRIPT_DIR/${test_name}.c"
            bin="$WORK_DIR/bins/${compiler}_${opt//-/}_${test_name}.exe"

            if ! compile_test "$compiler" "$src" "$opt" "$bin" 2>/dev/null; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name — compile failed"
                for combo in "${PASS_COMBOS[@]}"; do
                    label="${combo%%:*}"
                    for seed in $SEEDS; do
                        TOTAL=$((TOTAL + 1)); SKIPPED=$((SKIPPED + 1))
                        echo "$compiler,$opt,$test_name,$label,$seed,COMPILE_FAIL,-1" >> "$RESULTS_FILE"
                    done
                done
                continue
            fi

            local_exit=0
            timeout 30 "$bin" >/dev/null 2>&1 || local_exit=$?
            if [ "$local_exit" -ne 0 ]; then
                echo -e "    ${YELLOW}SKIP${NC} $test_name — original fails (exit=$local_exit)"
                for combo in "${PASS_COMBOS[@]}"; do
                    label="${combo%%:*}"
                    for seed in $SEEDS; do
                        TOTAL=$((TOTAL + 1)); SKIPPED=$((SKIPPED + 1))
                        echo "$compiler,$opt,$test_name,$label,$seed,ORIG_FAIL,$local_exit" >> "$RESULTS_FILE"
                    done
                done
                continue
            fi

            for combo in "${PASS_COMBOS[@]}"; do
                label="${combo%%:*}"
                disable="${combo#*:}"
                for seed in $SEEDS; do
                    run_test "$compiler" "$opt" "$test_name" "$label" "$disable" "$seed" "$bin"
                done
            done
        done
    done
done

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
{
    echo "============================================"
    echo "  Cobra Obfuscator Test Matrix Summary"
    echo "  $(date)"
    echo "============================================"
    echo ""
    echo "Total: $TOTAL  Passed: $PASSED  Failed: $FAILED  Skipped: $SKIPPED"
    echo ""

    echo "--- By Compiler ---"
    for compiler in $AVAILABLE_COMPILERS; do
        p=$(grep "^$compiler," "$RESULTS_FILE" | grep -c ",PASS," || true)
        f=$(grep "^$compiler," "$RESULTS_FILE" | grep ",FAIL,\|OBF_ERROR" | grep -vc "COMPILE_FAIL\|ORIG_FAIL" || true)
        s=$(grep "^$compiler," "$RESULTS_FILE" | grep -c "COMPILE_FAIL\|ORIG_FAIL\|SKIP" || true)
        echo "  $compiler: $p pass, $f fail, $s skip"
    done
    echo ""

    echo "--- By Optimization Level ---"
    for opt in -O0 -O1 -O2 -Os -O3; do
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
