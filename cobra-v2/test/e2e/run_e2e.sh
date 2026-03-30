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
