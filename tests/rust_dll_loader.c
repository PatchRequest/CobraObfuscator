#include <stdio.h>
#include <stdlib.h>
#include <windows.h>
#include <stdint.h>

typedef int32_t (*add_fn)(int32_t, int32_t);
typedef int64_t (*fibonacci_fn)(int32_t);
typedef uint64_t (*factorial_fn)(uint32_t);
typedef void (*sort_fn)(int32_t*, size_t);
typedef int64_t (*sum_fn)(const int32_t*, size_t);
typedef uint32_t (*hash_fn)(const uint8_t*, size_t);
typedef uint64_t (*gcd_fn)(uint64_t, uint64_t);
typedef int32_t (*is_prime_fn)(uint64_t);

int main(int argc, char *argv[]) {
    const char *dll_path = "rust_dll.dll";
    if (argc > 1) dll_path = argv[1];

    HMODULE hMod = LoadLibraryA(dll_path);
    if (!hMod) {
        fprintf(stderr, "Failed to load %s (error %lu)\n", dll_path, GetLastError());
        return 1;
    }

    int failures = 0;

    // rust_add
    add_fn add = (add_fn)GetProcAddress(hMod, "rust_add");
    if (!add) { fprintf(stderr, "Missing: rust_add\n"); return 1; }
    if (add(3, 7) != 10) { fprintf(stderr, "FAIL: rust_add(3,7)\n"); failures++; }
    if (add(-100, 100) != 0) { fprintf(stderr, "FAIL: rust_add(-100,100)\n"); failures++; }

    // rust_fibonacci
    fibonacci_fn fib = (fibonacci_fn)GetProcAddress(hMod, "rust_fibonacci");
    if (!fib) { fprintf(stderr, "Missing: rust_fibonacci\n"); return 1; }
    if (fib(0) != 0) { fprintf(stderr, "FAIL: rust_fibonacci(0)\n"); failures++; }
    if (fib(1) != 1) { fprintf(stderr, "FAIL: rust_fibonacci(1)\n"); failures++; }
    if (fib(10) != 55) { fprintf(stderr, "FAIL: rust_fibonacci(10)\n"); failures++; }
    if (fib(30) != 832040) { fprintf(stderr, "FAIL: rust_fibonacci(30)\n"); failures++; }

    // rust_factorial
    factorial_fn fact = (factorial_fn)GetProcAddress(hMod, "rust_factorial");
    if (!fact) { fprintf(stderr, "Missing: rust_factorial\n"); return 1; }
    if (fact(0) != 1) { fprintf(stderr, "FAIL: rust_factorial(0)\n"); failures++; }
    if (fact(5) != 120) { fprintf(stderr, "FAIL: rust_factorial(5)\n"); failures++; }
    if (fact(10) != 3628800) { fprintf(stderr, "FAIL: rust_factorial(10)\n"); failures++; }

    // rust_sort
    sort_fn sort = (sort_fn)GetProcAddress(hMod, "rust_sort");
    if (!sort) { fprintf(stderr, "Missing: rust_sort\n"); return 1; }
    int32_t arr[] = {9, 3, 7, 1, 5, 8, 2, 6, 4, 0};
    sort(arr, 10);
    for (int i = 0; i < 10; i++) {
        if (arr[i] != i) { fprintf(stderr, "FAIL: rust_sort arr[%d]=%d\n", i, arr[i]); failures++; break; }
    }

    // rust_sum
    sum_fn sum = (sum_fn)GetProcAddress(hMod, "rust_sum");
    if (!sum) { fprintf(stderr, "Missing: rust_sum\n"); return 1; }
    int32_t arr2[] = {1, 2, 3, 4, 5};
    if (sum(arr2, 5) != 15) { fprintf(stderr, "FAIL: rust_sum\n"); failures++; }
    if (sum(NULL, 0) != 0) { fprintf(stderr, "FAIL: rust_sum(NULL)\n"); failures++; }

    // rust_fnv_hash
    hash_fn hash = (hash_fn)GetProcAddress(hMod, "rust_fnv_hash");
    if (!hash) { fprintf(stderr, "Missing: rust_fnv_hash\n"); return 1; }
    uint32_t h1 = hash((const uint8_t*)"hello", 5);
    uint32_t h2 = hash((const uint8_t*)"hello", 5);
    if (h1 != h2) { fprintf(stderr, "FAIL: rust_fnv_hash determinism\n"); failures++; }
    uint32_t h3 = hash((const uint8_t*)"world", 5);
    if (h1 == h3) { fprintf(stderr, "FAIL: rust_fnv_hash collision\n"); failures++; }

    // rust_gcd
    gcd_fn gcd = (gcd_fn)GetProcAddress(hMod, "rust_gcd");
    if (!gcd) { fprintf(stderr, "Missing: rust_gcd\n"); return 1; }
    if (gcd(12, 8) != 4) { fprintf(stderr, "FAIL: rust_gcd(12,8)\n"); failures++; }
    if (gcd(100, 75) != 25) { fprintf(stderr, "FAIL: rust_gcd(100,75)\n"); failures++; }
    if (gcd(17, 13) != 1) { fprintf(stderr, "FAIL: rust_gcd(17,13)\n"); failures++; }

    // rust_is_prime
    is_prime_fn prime = (is_prime_fn)GetProcAddress(hMod, "rust_is_prime");
    if (!prime) { fprintf(stderr, "Missing: rust_is_prime\n"); return 1; }
    if (prime(0) != 0) { fprintf(stderr, "FAIL: rust_is_prime(0)\n"); failures++; }
    if (prime(1) != 0) { fprintf(stderr, "FAIL: rust_is_prime(1)\n"); failures++; }
    if (prime(2) != 1) { fprintf(stderr, "FAIL: rust_is_prime(2)\n"); failures++; }
    if (prime(17) != 1) { fprintf(stderr, "FAIL: rust_is_prime(17)\n"); failures++; }
    if (prime(100) != 0) { fprintf(stderr, "FAIL: rust_is_prime(100)\n"); failures++; }

    FreeLibrary(hMod);

    if (failures == 0) {
        printf("All Rust DLL tests passed!\n");
        return 0;
    } else {
        fprintf(stderr, "%d test(s) failed\n", failures);
        return 1;
    }
}
