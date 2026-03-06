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
# DLL tests:     C DLL (GCC) + Rust cdylib with loader validation
# Pass combos: all, cff-only, junk-only, dead-only, insn-only, cff+junk, etc.
# Seeds: 1..10
#
# Optimizations:
#   - Cached compilation: binaries only rebuilt when source changes
#   - Parallel execution via xargs -P (default: nproc jobs)
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COBRA="$PROJECT_DIR/target/release/cobra-obfuscator.exe"
WORK_DIR="/c/tmp/cobra_test_matrix"
RESULTS_FILE="$WORK_DIR/results.csv"
SUMMARY_FILE="$WORK_DIR/summary.txt"
BIN_CACHE="$WORK_DIR/bins"
OBF_DIR="$WORK_DIR/obf"
JOB_RESULTS_DIR="$WORK_DIR/job_results"
JOBLIST="$WORK_DIR/joblist.txt"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parallelism: use nproc or fallback, allow override via JOBS env var
MAX_JOBS="${JOBS:-$(nproc 2>/dev/null || echo 8)}"

# ---- Compiler detection ----
GCC="$(which gcc 2>/dev/null || true)"
CLANG="$(which clang 2>/dev/null || true)"
RUSTC="$(which rustc 2>/dev/null || true)"
CARGO="$(which cargo 2>/dev/null || true)"
GO="$(which go 2>/dev/null || true)"

# MSVC: detect via vcvarsall.bat
MSVC=""
VCVARSALL=""
for vs_path in \
    "/c/Program Files/Microsoft Visual Studio/2022/Community" \
    "/c/Program Files/Microsoft Visual Studio/2022/Professional" \
    "/c/Program Files/Microsoft Visual Studio/2022/Enterprise" \
    "/c/Program Files (x86)/Microsoft Visual Studio/2019/Community" \
    "/c/Program Files (x86)/Microsoft Visual Studio/2019/Professional"; do
    if [ -f "$vs_path/VC/Auxiliary/Build/vcvarsall.bat" ]; then
        VCVARSALL="$(cd "$vs_path/VC/Auxiliary/Build" && pwd -W)/vcvarsall.bat"
        break
    fi
done

if [ -n "$VCVARSALL" ]; then
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
echo "Parallel jobs: $MAX_JOBS"
echo ""

# ---- Setup ----
mkdir -p "$BIN_CACHE" "$OBF_DIR" "$JOB_RESULTS_DIR"

# MSVC bat wrapper
MSVC_BAT="$WORK_DIR/msvc_compile.bat"
if [ -n "$VCVARSALL" ]; then
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

# ---- Cached compilation helpers ----
needs_rebuild() {
    local bin="$1"
    shift
    [ ! -f "$bin" ] && return 0
    for src in "$@"; do
        if [ -f "$src" ]; then
            [ "$src" -nt "$bin" ] && return 0
        elif [ -d "$src" ]; then
            while IFS= read -r -d '' f; do
                [ "$f" -nt "$bin" ] && return 0
            done < <(find "$src" -type f -print0 2>/dev/null)
        fi
    done
    [ "$COBRA" -nt "$bin" ] && return 0
    return 1
}

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
            local win_out win_src win_bat
            win_out="$(cygpath -w "$out" 2>/dev/null || echo "$out")"
            win_src="$(cygpath -w "$src" 2>/dev/null || echo "$src")"
            win_bat="$(cygpath -w "$MSVC_BAT" 2>/dev/null || echo "$MSVC_BAT")"
            cmd.exe //C "$win_bat /Fe:$win_out $opt $win_src" >/dev/null 2>&1
            local base
            base="$(basename "$src" .c)"
            rm -f "${base}.obj" 2>/dev/null
            ;;
    esac
}

# ============================================================================
# Phase 1: Compile all test binaries (cached)
# ============================================================================
echo -e "${CYAN}===== Phase 1: Compile test binaries (cached) =====${NC}"
COMPILED=0
CACHED=0

declare -A COMPILER_OPTS
COMPILER_OPTS[gcc]="-O0 -O1 -O2 -Os -O3"
COMPILER_OPTS[clang]="-O0 -O1 -O2 -Os -O3"
COMPILER_OPTS[msvc]="/Od /O1 /O2"

AVAILABLE_COMPILERS=""
[ -n "$GCC" ]   && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS gcc"
[ -n "$CLANG" ] && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS clang"
[ -n "$MSVC" ]  && AVAILABLE_COMPILERS="$AVAILABLE_COMPILERS msvc"

for compiler in $AVAILABLE_COMPILERS; do
    for opt in ${COMPILER_OPTS[$compiler]}; do
        for test_name in $ALL_C_TESTS; do
            src="$SCRIPT_DIR/${test_name}.c"
            safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
            bin="$BIN_CACHE/${compiler}_${safe_opt}_${test_name}.exe"

            if ! needs_rebuild "$bin" "$src"; then
                CACHED=$((CACHED + 1))
                continue
            fi

            if compile_test "$compiler" "$src" "$opt" "$bin" 2>/dev/null; then
                COMPILED=$((COMPILED + 1))
            else
                rm -f "$bin"
            fi
        done
    done
done

if [ -n "$CARGO" ]; then
    for test_name in $ALL_RUST_TESTS; do
        test_dir="$SCRIPT_DIR/$test_name"
        [ ! -f "$test_dir/Cargo.toml" ] && continue

        for profile in debug release; do
            bin="$BIN_CACHE/rust_${profile}_${test_name}.exe"

            if ! needs_rebuild "$bin" "$test_dir/src" "$test_dir/Cargo.toml"; then
                CACHED=$((CACHED + 1))
                continue
            fi

            if [ "$profile" = "release" ]; then
                cargo_flag="--release"
                target_subdir="release"
            else
                cargo_flag=""
                target_subdir="debug"
            fi

            if (cd "$test_dir" && cargo build $cargo_flag 2>/dev/null); then
                built_exe="$test_dir/target/$target_subdir/${test_name}.exe"
                if [ ! -f "$built_exe" ]; then
                    alt_name="${test_name//-/_}"
                    built_exe="$test_dir/target/$target_subdir/${alt_name}.exe"
                fi
                if [ -f "$built_exe" ]; then
                    cp "$built_exe" "$bin"
                    COMPILED=$((COMPILED + 1))
                fi
            else
                rm -f "$bin"
            fi
        done
    done
fi

if [ -n "$GO" ]; then
    for test_name in $ALL_GO_TESTS; do
        test_dir="$SCRIPT_DIR/$test_name"
        [ ! -f "$test_dir/main.go" ] && continue

        bin="$BIN_CACHE/go_default_${test_name}.exe"

        if ! needs_rebuild "$bin" "$test_dir"; then
            CACHED=$((CACHED + 1))
            continue
        fi

        if (cd "$test_dir" && GOOS=windows GOARCH=amd64 go build -o "$bin" . 2>/dev/null); then
            COMPILED=$((COMPILED + 1))
        else
            rm -f "$bin"
        fi
    done
fi

# DLLs
if [ -n "$GCC" ]; then
    dll_src="$SCRIPT_DIR/test_dll.c"
    loader_src="$SCRIPT_DIR/test_dll_loader.c"
    dll_bin="$BIN_CACHE/gcc_dll_test_dll.dll"
    loader_bin="$BIN_CACHE/gcc_dll_loader.exe"

    if needs_rebuild "$dll_bin" "$dll_src"; then
        if "$GCC" -shared -o "$dll_bin" "$dll_src" -lkernel32 2>/dev/null; then
            COMPILED=$((COMPILED + 1))
        else
            rm -f "$dll_bin"
        fi
    else
        CACHED=$((CACHED + 1))
    fi

    if needs_rebuild "$loader_bin" "$loader_src"; then
        if "$GCC" -o "$loader_bin" "$loader_src" -lkernel32 2>/dev/null; then
            COMPILED=$((COMPILED + 1))
        else
            rm -f "$loader_bin"
        fi
    else
        CACHED=$((CACHED + 1))
    fi
fi

if [ -n "$CARGO" ]; then
    rust_dll_dir="$SCRIPT_DIR/rust_dll"
    if [ -f "$rust_dll_dir/Cargo.toml" ]; then
        rust_dll_bin="$BIN_CACHE/rust_dll_release.dll"
        if needs_rebuild "$rust_dll_bin" "$rust_dll_dir/src" "$rust_dll_dir/Cargo.toml"; then
            if (cd "$rust_dll_dir" && cargo build --release 2>/dev/null); then
                built_dll="$rust_dll_dir/target/release/rust_dll.dll"
                [ ! -f "$built_dll" ] && built_dll="$(find "$rust_dll_dir/target/release" -name '*.dll' -print -quit 2>/dev/null || true)"
                if [ -n "$built_dll" ] && [ -f "$built_dll" ]; then
                    cp "$built_dll" "$rust_dll_bin"
                    COMPILED=$((COMPILED + 1))
                fi
            else
                rm -f "$rust_dll_bin"
            fi
        else
            CACHED=$((CACHED + 1))
        fi

        rust_dll_loader_src="$SCRIPT_DIR/rust_dll_loader.c"
        rust_dll_loader_bin="$BIN_CACHE/rust_dll_loader.exe"
        if [ -f "$rust_dll_loader_src" ] && [ -n "$GCC" ]; then
            if needs_rebuild "$rust_dll_loader_bin" "$rust_dll_loader_src"; then
                if "$GCC" -o "$rust_dll_loader_bin" "$rust_dll_loader_src" -lkernel32 2>/dev/null; then
                    COMPILED=$((COMPILED + 1))
                else
                    rm -f "$rust_dll_loader_bin"
                fi
            else
                CACHED=$((CACHED + 1))
            fi
        fi
    fi
fi

echo -e "  Compiled: $COMPILED, Cached: $CACHED"
echo ""

# ============================================================================
# Phase 2: Validate originals in parallel via xargs
# ============================================================================
echo -e "${CYAN}===== Phase 2: Validate original binaries =====${NC}"

VALID_DIR="$WORK_DIR/valid"
mkdir -p "$VALID_DIR"
rm -f "$VALID_DIR"/*.status 2>/dev/null

# Build list of binaries to validate: key|binary_path
VALIDATE_LIST="$WORK_DIR/validate_list.txt"
> "$VALIDATE_LIST"

for compiler in $AVAILABLE_COMPILERS; do
    for opt in ${COMPILER_OPTS[$compiler]}; do
        for test_name in $ALL_C_TESTS; do
            safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
            bin="$BIN_CACHE/${compiler}_${safe_opt}_${test_name}.exe"
            [ -f "$bin" ] && echo "${compiler}_${safe_opt}_${test_name}|$bin" >> "$VALIDATE_LIST"
        done
    done
done

if [ -n "$CARGO" ]; then
    for test_name in $ALL_RUST_TESTS; do
        for profile in debug release; do
            bin="$BIN_CACHE/rust_${profile}_${test_name}.exe"
            [ -f "$bin" ] && echo "rust_${profile}_${test_name}|$bin" >> "$VALIDATE_LIST"
        done
    done
fi

if [ -n "$GO" ]; then
    for test_name in $ALL_GO_TESTS; do
        bin="$BIN_CACHE/go_default_${test_name}.exe"
        [ -f "$bin" ] && echo "go_default_${test_name}|$bin" >> "$VALIDATE_LIST"
    done
fi

VALIDATE_COUNT=$(wc -l < "$VALIDATE_LIST")
echo "  Validating $VALIDATE_COUNT binaries ($MAX_JOBS parallel)..."

# Validate in parallel using xargs
export VALID_DIR
cat "$VALIDATE_LIST" | xargs -P "$MAX_JOBS" -I{} bash -c '
    key="${1%%|*}"
    bin="${1#*|}"
    exit_code=0
    timeout 30 "$bin" >/dev/null 2>&1 || exit_code=$?
    echo "$exit_code" > "$VALID_DIR/${key}.status"
' _ {}

VALID_OK=0
VALID_FAIL=0
for f in "$VALID_DIR"/*.status; do
    [ ! -f "$f" ] && continue
    code="$(cat "$f")"
    if [ "$code" -eq 0 ]; then
        VALID_OK=$((VALID_OK + 1))
    else
        VALID_FAIL=$((VALID_FAIL + 1))
    fi
done
echo -e "  ${GREEN}Valid: $VALID_OK${NC}, ${YELLOW}Failed: $VALID_FAIL${NC}"
echo ""

# ============================================================================
# Phase 3: Build job list + execute all tests via xargs -P
# ============================================================================
echo -e "${CYAN}===== Phase 3: Obfuscate + validate ($MAX_JOBS parallel jobs) =====${NC}"

rm -f "$JOB_RESULTS_DIR"/*.result 2>/dev/null
> "$JOBLIST"

# Helper: write skip results directly + append to joblist skip count
write_skips() {
    local compiler_name="$1" opt="$2" test_name="$3" reason="$4" exit_code="$5"
    local safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
    for combo in "${PASS_COMBOS[@]}"; do
        label="${combo%%:*}"
        for seed in $SEEDS; do
            local out_name="${compiler_name}_${safe_opt}_${test_name}_${label}_s${seed}"
            echo "$compiler_name,$opt,$test_name,$label,$seed,${reason},${exit_code}" > "$JOB_RESULTS_DIR/${out_name}.result"
        done
    done
}

# Helper: add all combos for a binary to the job list
# Format: compiler|opt|test_name|pass_label|disable_flags|seed|input_exe
add_all_combos() {
    local compiler_name="$1" opt="$2" test_name="$3" bin="$4"
    for combo in "${PASS_COMBOS[@]}"; do
        label="${combo%%:*}"
        disable="${combo#*:}"
        for seed in $SEEDS; do
            echo "$compiler_name|$opt|$test_name|$label|$disable|$seed|$bin" >> "$JOBLIST"
        done
    done
}

# ---- C tests ----
for compiler in $AVAILABLE_COMPILERS; do
    for opt in ${COMPILER_OPTS[$compiler]}; do
        for test_name in $ALL_C_TESTS; do
            safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
            key="${compiler}_${safe_opt}_${test_name}"
            bin="$BIN_CACHE/${key}.exe"

            if [ ! -f "$bin" ]; then
                write_skips "$compiler" "$opt" "$test_name" "COMPILE_FAIL" "-1"
                continue
            fi

            status_file="$VALID_DIR/${key}.status"
            if [ ! -f "$status_file" ] || [ "$(cat "$status_file")" -ne 0 ]; then
                local_exit="$(cat "$status_file" 2>/dev/null || echo -1)"
                write_skips "$compiler" "$opt" "$test_name" "ORIG_FAIL" "$local_exit"
                continue
            fi

            add_all_combos "$compiler" "$opt" "$test_name" "$bin"
        done
    done
done

# ---- Rust tests ----
if [ -n "$CARGO" ]; then
    for test_name in $ALL_RUST_TESTS; do
        for profile in debug release; do
            key="rust_${profile}_${test_name}"
            bin="$BIN_CACHE/${key}.exe"

            if [ ! -f "$bin" ]; then
                write_skips "rust" "$profile" "$test_name" "COMPILE_FAIL" "-1"
                continue
            fi

            status_file="$VALID_DIR/${key}.status"
            if [ ! -f "$status_file" ] || [ "$(cat "$status_file")" -ne 0 ]; then
                local_exit="$(cat "$status_file" 2>/dev/null || echo -1)"
                write_skips "rust" "$profile" "$test_name" "ORIG_FAIL" "$local_exit"
                continue
            fi

            add_all_combos "rust" "$profile" "$test_name" "$bin"
        done
    done
fi

# ---- Go tests ----
if [ -n "$GO" ]; then
    for test_name in $ALL_GO_TESTS; do
        key="go_default_${test_name}"
        bin="$BIN_CACHE/${key}.exe"

        if [ ! -f "$bin" ]; then
            write_skips "go" "default" "$test_name" "COMPILE_FAIL" "-1"
            continue
        fi

        status_file="$VALID_DIR/${key}.status"
        if [ ! -f "$status_file" ] || [ "$(cat "$status_file")" -ne 0 ]; then
            local_exit="$(cat "$status_file" 2>/dev/null || echo -1)"
            write_skips "go" "default" "$test_name" "ORIG_FAIL" "$local_exit"
            continue
        fi

        add_all_combos "go" "default" "$test_name" "$bin"
    done
fi

# ---- DLL tests ----
if [ -n "$GCC" ]; then
    dll_bin="$BIN_CACHE/gcc_dll_test_dll.dll"
    loader_bin="$BIN_CACHE/gcc_dll_loader.exe"
    if [ -f "$dll_bin" ] && [ -f "$loader_bin" ]; then
        # Validate original DLL
        tmp_dll_dir="$WORK_DIR/dll_validate_gcc"
        mkdir -p "$tmp_dll_dir"
        cp "$dll_bin" "$tmp_dll_dir/test_dll.dll"
        dll_exit=0
        (cd "$tmp_dll_dir" && timeout 30 "$loader_bin" >/dev/null 2>&1) || dll_exit=$?
        rm -rf "$tmp_dll_dir"

        if [ "$dll_exit" -ne 0 ]; then
            write_skips "gcc_dll" "default" "test_dll" "ORIG_FAIL" "$dll_exit"
        else
            # DLL jobs: format is dll|compiler|test_name|label|disable|seed|dll_bin|loader_bin|dll_filename
            for combo in "${PASS_COMBOS[@]}"; do
                label="${combo%%:*}"
                disable="${combo#*:}"
                for seed in $SEEDS; do
                    echo "DLL|gcc_dll|default|test_dll|$label|$disable|$seed|$dll_bin|$loader_bin|test_dll.dll" >> "$JOBLIST"
                done
            done
        fi
    fi
fi

if [ -n "$CARGO" ]; then
    rust_dll_bin="$BIN_CACHE/rust_dll_release.dll"
    rust_dll_loader_bin="$BIN_CACHE/rust_dll_loader.exe"
    if [ -f "$rust_dll_bin" ] && [ -f "$rust_dll_loader_bin" ]; then
        tmp_dll_dir="$WORK_DIR/dll_validate_rust"
        mkdir -p "$tmp_dll_dir"
        cp "$rust_dll_bin" "$tmp_dll_dir/rust_dll.dll"
        dll_exit=0
        (cd "$tmp_dll_dir" && timeout 30 "$rust_dll_loader_bin" >/dev/null 2>&1) || dll_exit=$?
        rm -rf "$tmp_dll_dir"

        if [ "$dll_exit" -ne 0 ]; then
            write_skips "rust_dll" "release" "rust_dll" "ORIG_FAIL" "$dll_exit"
        else
            for combo in "${PASS_COMBOS[@]}"; do
                label="${combo%%:*}"
                disable="${combo#*:}"
                for seed in $SEEDS; do
                    echo "DLL|rust_dll|release|rust_dll|$label|$disable|$seed|$rust_dll_bin|$rust_dll_loader_bin|rust_dll.dll" >> "$JOBLIST"
                done
            done
        fi
    fi
fi

JOB_COUNT=$(wc -l < "$JOBLIST")
echo "  Queued $JOB_COUNT test jobs"
echo ""

# ---- Execute all jobs via xargs -P ----
export COBRA OBF_DIR JOB_RESULTS_DIR

cat "$JOBLIST" | xargs -P "$MAX_JOBS" -I{} bash -c '
    IFS="|" read -r type_or_compiler f2 f3 f4 f5 f6 f7 f8 f9 f10 <<< "$1"

    if [ "$type_or_compiler" = "DLL" ]; then
        # DLL job: DLL|compiler|opt|test_name|label|disable|seed|dll_bin|loader_bin|dll_filename
        compiler_name="$f2"
        opt="$f3"
        test_name="$f4"
        pass_label="$f5"
        disable_flags="$f6"
        seed="$f7"
        dll_bin="$f8"
        loader_bin="$f9"
        dll_filename="$f10"

        safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
        out_name="${compiler_name}_${safe_opt}_${test_name}_${pass_label}_s${seed}"
        obf_dll="$OBF_DIR/${out_name}.dll"
        result_file="$JOB_RESULTS_DIR/${out_name}.result"

        cmd="$COBRA -i $dll_bin -o $obf_dll --seed $seed --encrypt-strings --fluctuate"
        [ -n "$disable_flags" ] && cmd="$cmd --disable $disable_flags"

        if ! eval "$cmd" >/dev/null 2>&1; then
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,OBF_ERROR,-1" > "$result_file"
            exit 0
        fi

        tmp_dir="$OBF_DIR/dll_test_${out_name}_$$"
        mkdir -p "$tmp_dir"
        cp "$obf_dll" "$tmp_dir/$dll_filename"
        exit_code=0
        (cd "$tmp_dir" && timeout 30 "$loader_bin" >/dev/null 2>&1) || exit_code=$?
        rm -rf "$tmp_dir" "$obf_dll"

        if [ "$exit_code" -eq 0 ]; then
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,PASS,0" > "$result_file"
        else
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,FAIL,$exit_code" > "$result_file"
        fi
    else
        # EXE job: compiler|opt|test_name|label|disable|seed|input_exe
        compiler_name="$type_or_compiler"
        opt="$f2"
        test_name="$f3"
        pass_label="$f4"
        disable_flags="$f5"
        seed="$f6"
        input_exe="$f7"

        safe_opt="${opt#-}"; safe_opt="${safe_opt#/}"
        out_name="${compiler_name}_${safe_opt}_${test_name}_${pass_label}_s${seed}"
        out_exe="$OBF_DIR/${out_name}.exe"
        result_file="$JOB_RESULTS_DIR/${out_name}.result"

        cmd="$COBRA -i $input_exe -o $out_exe --seed $seed --encrypt-strings --fluctuate"
        [ -n "$disable_flags" ] && cmd="$cmd --disable $disable_flags"

        if ! eval "$cmd" >/dev/null 2>&1; then
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,OBF_ERROR,-1" > "$result_file"
            exit 0
        fi

        exit_code=0
        timeout 30 "$out_exe" >/dev/null 2>&1 || exit_code=$?
        rm -f "$out_exe"

        if [ "$exit_code" -eq 0 ]; then
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,PASS,0" > "$result_file"
        else
            echo "$compiler_name,$opt,$test_name,$pass_label,$seed,FAIL,$exit_code" > "$result_file"
        fi
    fi
' _ {}

# ============================================================================
# Phase 4: Collect results
# ============================================================================
echo -e "${CYAN}===== Phase 4: Collect results =====${NC}"

TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

for result_file in "$JOB_RESULTS_DIR"/*.result; do
    [ ! -f "$result_file" ] && continue
    line="$(cat "$result_file")"
    echo "$line" >> "$RESULTS_FILE"
    TOTAL=$((TOTAL + 1))

    case "$line" in
        *",PASS,"*)      PASSED=$((PASSED + 1)) ;;
        *",COMPILE_FAIL,"*|*",ORIG_FAIL,"*|*",SKIP,"*)  SKIPPED=$((SKIPPED + 1)) ;;
        *)               FAILED=$((FAILED + 1))
                         echo -e "  ${RED}FAIL${NC} $line"
                         ;;
    esac
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
ALL_TESTS="$ALL_C_TESTS $ALL_RUST_TESTS $ALL_GO_TESTS test_dll rust_dll"
ALL_OPT_LEVELS="-O0 -O1 -O2 -Os -O3 /Od /O1 /O2 debug release default"
ALL_COMPILERS="$AVAILABLE_COMPILERS"
[ -n "$CARGO" ] && ALL_COMPILERS="$ALL_COMPILERS rust rust_dll"
[ -n "$GO" ]    && ALL_COMPILERS="$ALL_COMPILERS go"
[ -n "$GCC" ]   && ALL_COMPILERS="$ALL_COMPILERS gcc_dll"

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
        [ $((p + f + s)) -gt 0 ] && echo "  $compiler: $p pass, $f fail, $s skip"
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
        [ $((p + f + s)) -gt 0 ] && echo "  $test_name: $p pass, $f fail, $s skip"
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
