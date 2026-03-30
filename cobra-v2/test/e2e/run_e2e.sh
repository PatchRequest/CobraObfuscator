#!/bin/bash
set -e

COBRA="$(dirname "$0")/../../build/cobra-v2"
CLANG="/opt/homebrew/opt/llvm/bin/clang"
CLANGXX="/opt/homebrew/opt/llvm/bin/clang++"
TESTDIR="$(dirname "$0")"
TMPDIR="/tmp/cobra_e2e"
TIMEOUT=120
mkdir -p "$TMPDIR"

PASS=0
FAIL=0
SKIP=0
TOTAL=0
FAILURES=""

# Portable timeout (macOS lacks GNU timeout)
run_with_timeout() {
    local secs="$1"; shift
    "$@" &
    local pid=$!
    (sleep "$secs" && kill "$pid" 2>/dev/null) &
    local watchdog=$!
    wait "$pid" 2>/dev/null
    local rc=$?
    kill "$watchdog" 2>/dev/null
    wait "$watchdog" 2>/dev/null
    return $rc
}

run_c_test() {
    local name="$1" src="$2" expected="$3" passes="${4:-all}" seed="${5:-42}" iterations="${6:-1}"
    TOTAL=$((TOTAL + 1))
    echo -n "  $name ($passes s=$seed i=$iterations)... "

    if ! run_with_timeout "$TIMEOUT" $CLANG -emit-llvm -c "$src" -o "$TMPDIR/${name}.bc" 2>/dev/null; then
        echo "SKIP (compile)"; SKIP=$((SKIP + 1)); return; fi
    if ! run_with_timeout "$TIMEOUT" $COBRA "$TMPDIR/${name}.bc" -o "$TMPDIR/${name}_obf.bc" \
        --passes "$passes" --seed "$seed" --iterations "$iterations" 2>/dev/null; then
        echo "FAIL (obfuscator)"; FAIL=$((FAIL + 1))
        FAILURES="$FAILURES\n  CRASH: $name ($passes s=$seed)"; return; fi
    if ! run_with_timeout "$TIMEOUT" $CLANG "$TMPDIR/${name}_obf.bc" -o "$TMPDIR/${name}" -lm 2>/dev/null; then
        echo "FAIL (link)"; FAIL=$((FAIL + 1))
        FAILURES="$FAILURES\n  LINK: $name ($passes s=$seed)"; return; fi

    local actual; actual=$(run_with_timeout 10 "$TMPDIR/${name}" 2>&1) || true
    if [ "$actual" = "$expected" ]; then
        echo "PASS"; PASS=$((PASS + 1))
    else
        echo "FAIL (output)"
        FAIL=$((FAIL + 1)); FAILURES="$FAILURES\n  OUTPUT: $name ($passes s=$seed)"
    fi
}

run_cpp_test() {
    local name="$1" src="$2" expected="$3" passes="${4:-all}" seed="${5:-42}"
    TOTAL=$((TOTAL + 1))
    echo -n "  $name ($passes s=$seed)... "

    if ! run_with_timeout "$TIMEOUT" $CLANGXX -emit-llvm -c "$src" -o "$TMPDIR/${name}.bc" 2>/dev/null; then
        echo "SKIP (compile)"; SKIP=$((SKIP + 1)); return; fi
    if ! run_with_timeout 60 $COBRA "$TMPDIR/${name}.bc" -o "$TMPDIR/${name}_obf.bc" \
        --passes "$passes" --seed "$seed" 2>/dev/null; then
        echo "FAIL (obfuscator)"; FAIL=$((FAIL + 1))
        FAILURES="$FAILURES\n  CRASH: $name ($passes s=$seed)"; return; fi
    if ! run_with_timeout 60 $CLANGXX "$TMPDIR/${name}_obf.bc" -o "$TMPDIR/${name}" 2>/dev/null; then
        echo "FAIL (link/codegen timeout)"; FAIL=$((FAIL + 1))
        FAILURES="$FAILURES\n  LINK: $name ($passes s=$seed)"; return; fi

    local actual; actual=$(run_with_timeout 10 "$TMPDIR/${name}" 2>&1) || true
    if [ "$actual" = "$expected" ]; then
        echo "PASS"; PASS=$((PASS + 1))
    else
        echo "FAIL (output)"
        FAIL=$((FAIL + 1)); FAILURES="$FAILURES\n  OUTPUT: $name ($passes s=$seed)"
    fi
}

# ── Expected outputs ─────────────────────────────────────────────────

E_ARITH="add=13 sub=7 xor=9"
E_CF="pos=1 neg=-1 zero=0"
E_RECURSION="fib(10)=55"
E_STRINGS="combined=Hello World len=11"
E_LOOPS="for=55 while=55 dowhile=55 nested=66"
E_SWITCH=$'d0=Sun d3=Wed d6=Sat d9=???\ng100=4 g85=3 g72=2 g55=0'
E_STRUCTS=$'add=(4,6) dot=11\nlistsum=15'
E_FPTR=$'apply: add=13 sub=7 mul=30\nreduce: sum=15 product=120'
E_BITWISE=$'pop(0xFF)=8 pop(0x1234)=5\nrev(1)=2147483648 rev(0x80000000)=1\npow2: 1=1 4=1 6=0 128=1\nnext: 3->4 5->8 17->32'
E_MATH=$'sqrt(144)=12 sqrt(2)=1.4142\ngcd(48,18)=6 gcd(100,75)=25\nfact(10)=3628800 fact(12)=479001600\nprime: 2=1 17=1 100=0 997=1'
E_ARRAYS=$'sorted=123456789\nfind7=6 find10=-1\nmat=[19,22;43,50]'
E_VARARGS="sum=15 max=9"
E_CPP_CLASS=$'Rect: area=15\nCircle: area=48\nTri: area=12'
E_CPP_TMPL=$'max(3,7)=7 max(2.5,1.5)=2.5\nmin(3,7)=3 min(2.5,1.5)=1.5\nsize=5 sum=150 max=50'
E_CPP_EXC=$'div(10,3)=3 sqrt(16)=4\ncaught: div by zero\ncaught: negative\nnested=20'

ALL_PASSES=(insn-substitution mba constant-unfold dead-code bogus-cf junk-insertion cff string-encrypt func-merge-split indirect-branch)

# C++ safe passes (dead-code/bogus-cf/junk-insertion cause codegen explosion on C++ IR)
CPP_SAFE_PASSES=(insn-substitution mba constant-unfold cff string-encrypt func-merge-split indirect-branch)

echo "================================================================"
echo "    CobraObfuscator v2 — Comprehensive E2E Test Suite"
echo "================================================================"
echo ""

# ══════════════════════════════════════════════════════════════
# SECTION 1: C programs × individual passes
# ══════════════════════════════════════════════════════════════

echo "--- Section 1: C programs x individual passes ---"

C_NAMES=(arith controlflow recursion strings loops switch structs fptr bitwise math arrays varargs)
C_SRCS=("$TESTDIR/arith.c" "$TESTDIR/controlflow.c" "$TESTDIR/recursion.c" "$TESTDIR/strings.c" "$TESTDIR/loops.c" "$TESTDIR/switch.c" "$TESTDIR/structs.c" "$TESTDIR/function_pointers.c" "$TESTDIR/bitwise.c" "$TESTDIR/math_heavy.c" "$TESTDIR/arrays.c" "$TESTDIR/varargs.c")
C_EXPECTS=("$E_ARITH" "$E_CF" "$E_RECURSION" "$E_STRINGS" "$E_LOOPS" "$E_SWITCH" "$E_STRUCTS" "$E_FPTR" "$E_BITWISE" "$E_MATH" "$E_ARRAYS" "$E_VARARGS")

for i in "${!C_NAMES[@]}"; do
    echo "  [${C_NAMES[$i]}]"
    for pass in "${ALL_PASSES[@]}"; do
        ptag="${pass//-/_}"
        run_c_test "${C_NAMES[$i]}_${ptag}" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "$pass"
    done
done

# ══════════════════════════════════════════════════════════════
# SECTION 2: C programs × all passes combined
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 2: C programs x all passes ---"

for i in "${!C_NAMES[@]}"; do
    run_c_test "${C_NAMES[$i]}_all" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "all"
done

# ══════════════════════════════════════════════════════════════
# SECTION 3: Multiple seeds
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 3: Multiple seeds (all passes) ---"

for seed in 1 123 999 7777; do
    for i in 0 1 4 8 10; do  # arith controlflow loops bitwise arrays
        run_c_test "${C_NAMES[$i]}_s${seed}" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "all" "$seed"
    done
done

# ══════════════════════════════════════════════════════════════
# SECTION 4: Multiple iterations
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 4: Multiple iterations (iter=2, all passes) ---"

for i in 0 1 2 4 8; do  # arith controlflow recursion loops bitwise
    run_c_test "${C_NAMES[$i]}_iter2" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "all" 42 2
done

# ══════════════════════════════════════════════════════════════
# SECTION 5: Pairwise pass combinations
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 5: Pairwise/triple pass combos ---"

PAIRS=(
    "cff,string-encrypt"
    "cff,dead-code"
    "cff,bogus-cf"
    "cff,junk-insertion"
    "cff,mba"
    "cff,insn-substitution"
    "cff,constant-unfold"
    "mba,insn-substitution"
    "mba,constant-unfold"
    "dead-code,bogus-cf"
    "dead-code,junk-insertion"
    "string-encrypt,mba"
    "string-encrypt,constant-unfold"
    "func-merge-split,indirect-branch"
    "func-merge-split,cff"
    "indirect-branch,cff"
    "insn-substitution,constant-unfold,mba"
    "cff,dead-code,bogus-cf,junk-insertion"
    "string-encrypt,cff,mba"
)

for pair in "${PAIRS[@]}"; do
    ptag="${pair//,/_}"
    ptag="${ptag//-/}"
    for i in 0 4 8 10; do  # arith loops bitwise arrays
        run_c_test "${C_NAMES[$i]}_${ptag}" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "$pair"
    done
done

# ══════════════════════════════════════════════════════════════
# SECTION 6: C++ programs (safe passes only)
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 6: C++ programs x safe passes ---"

CPP_NAMES=(cpp_class cpp_tmpl cpp_exc)
CPP_SRCS=("$TESTDIR/cpp_classes.cpp" "$TESTDIR/cpp_templates.cpp" "$TESTDIR/cpp_exceptions.cpp")
CPP_EXPECTS=("$E_CPP_CLASS" "$E_CPP_TMPL" "$E_CPP_EXC")

for i in "${!CPP_NAMES[@]}"; do
    echo "  [${CPP_NAMES[$i]}]"
    for pass in "${CPP_SAFE_PASSES[@]}"; do
        ptag="${pass//-/_}"
        run_cpp_test "${CPP_NAMES[$i]}_${ptag}" "${CPP_SRCS[$i]}" "${CPP_EXPECTS[$i]}" "$pass"
    done
done

# C++ with all safe passes combined
echo ""
echo "--- Section 7: C++ all safe passes + multiple seeds ---"

CPP_ALL_SAFE="insn-substitution,mba,constant-unfold,cff,string-encrypt,func-merge-split,indirect-branch"
for i in "${!CPP_NAMES[@]}"; do
    for seed in 42 123 999; do
        run_cpp_test "${CPP_NAMES[$i]}_allsafe_s${seed}" "${CPP_SRCS[$i]}" "${CPP_EXPECTS[$i]}" "$CPP_ALL_SAFE" "$seed"
    done
done

# ══════════════════════════════════════════════════════════════
# SECTION 8: Stress — all passes, iter=2, varied seeds
# ══════════════════════════════════════════════════════════════

echo ""
echo "--- Section 8: Stress (all passes, iter=2, varied seeds) ---"

for seed in 1 42 100 555 9999; do
    for i in 0 4 5 8 10 11; do  # arith loops switch bitwise arrays math
        run_c_test "${C_NAMES[$i]}_stress_s${seed}" "${C_SRCS[$i]}" "${C_EXPECTS[$i]}" "all" "$seed" 2
    done
done

# ══════════════════════════════════════════════════════════════
# RESULTS
# ══════════════════════════════════════════════════════════════

echo ""
echo "================================================================"
printf "  Results: %d passed, %d failed, %d skipped / %d total\n" "$PASS" "$FAIL" "$SKIP" "$TOTAL"
echo "================================================================"

if [ -n "$FAILURES" ]; then
    echo ""
    echo "Failures:"
    echo -e "$FAILURES"
fi

[ "$FAIL" -eq 0 ] || exit 1
