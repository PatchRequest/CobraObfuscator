#include <stdio.h>
#include <stdlib.h>
#include <windows.h>
#include <stdint.h>

typedef int (*add_numbers_fn)(int, int);
typedef int (*fibonacci_fn)(int);
typedef uint32_t (*xor_hash_fn)(const char*, int);
typedef void (*bubble_sort_fn)(int*, int);
typedef int (*matrix_trace_fn)(int[4][4]);
typedef int (*string_length_fn)(const char*);

int main(int argc, char *argv[]) {
    const char *dll_path = "test_dll.dll";
    if (argc > 1) dll_path = argv[1];

    HMODULE hMod = LoadLibraryA(dll_path);
    if (!hMod) {
        fprintf(stderr, "Failed to load %s (error %lu)\n", dll_path, GetLastError());
        return 1;
    }

    int failures = 0;

    // Test add_numbers
    add_numbers_fn add_fn = (add_numbers_fn)GetProcAddress(hMod, "add_numbers");
    if (!add_fn) { fprintf(stderr, "Missing: add_numbers\n"); return 1; }
    if (add_fn(3, 7) != 10) { fprintf(stderr, "FAIL: add_numbers(3,7) != 10\n"); failures++; }
    if (add_fn(-5, 5) != 0) { fprintf(stderr, "FAIL: add_numbers(-5,5) != 0\n"); failures++; }
    if (add_fn(0, 0) != 0) { fprintf(stderr, "FAIL: add_numbers(0,0) != 0\n"); failures++; }

    // Test fibonacci
    fibonacci_fn fib_fn = (fibonacci_fn)GetProcAddress(hMod, "fibonacci");
    if (!fib_fn) { fprintf(stderr, "Missing: fibonacci\n"); return 1; }
    if (fib_fn(0) != 0) { fprintf(stderr, "FAIL: fibonacci(0) != 0\n"); failures++; }
    if (fib_fn(1) != 1) { fprintf(stderr, "FAIL: fibonacci(1) != 1\n"); failures++; }
    if (fib_fn(10) != 55) { fprintf(stderr, "FAIL: fibonacci(10) != 55\n"); failures++; }
    if (fib_fn(20) != 6765) { fprintf(stderr, "FAIL: fibonacci(20) != 6765\n"); failures++; }

    // Test xor_hash
    xor_hash_fn hash_fn = (xor_hash_fn)GetProcAddress(hMod, "xor_hash");
    if (!hash_fn) { fprintf(stderr, "Missing: xor_hash\n"); return 1; }
    uint32_t h1 = hash_fn("hello", 5);
    uint32_t h2 = hash_fn("hello", 5);
    if (h1 != h2) { fprintf(stderr, "FAIL: xor_hash determinism\n"); failures++; }
    uint32_t h3 = hash_fn("world", 5);
    if (h1 == h3) { fprintf(stderr, "FAIL: xor_hash collision\n"); failures++; }

    // Test bubble_sort
    bubble_sort_fn sort_fn = (bubble_sort_fn)GetProcAddress(hMod, "bubble_sort");
    if (!sort_fn) { fprintf(stderr, "Missing: bubble_sort\n"); return 1; }
    int arr[] = {5, 3, 8, 1, 9, 2, 7, 4, 6, 0};
    sort_fn(arr, 10);
    for (int i = 0; i < 10; i++) {
        if (arr[i] != i) { fprintf(stderr, "FAIL: bubble_sort arr[%d]=%d\n", i, arr[i]); failures++; break; }
    }

    // Test matrix_trace
    matrix_trace_fn trace_fn = (matrix_trace_fn)GetProcAddress(hMod, "matrix_trace");
    if (!trace_fn) { fprintf(stderr, "Missing: matrix_trace\n"); return 1; }
    int mat[4][4] = {
        {1, 0, 0, 0},
        {0, 2, 0, 0},
        {0, 0, 3, 0},
        {0, 0, 0, 4}
    };
    if (trace_fn(mat) != 10) { fprintf(stderr, "FAIL: matrix_trace != 10\n"); failures++; }

    // Test string_length
    string_length_fn strlen_fn = (string_length_fn)GetProcAddress(hMod, "string_length");
    if (!strlen_fn) { fprintf(stderr, "Missing: string_length\n"); return 1; }
    if (strlen_fn("") != 0) { fprintf(stderr, "FAIL: string_length('') != 0\n"); failures++; }
    if (strlen_fn("test") != 4) { fprintf(stderr, "FAIL: string_length('test') != 4\n"); failures++; }
    if (strlen_fn("hello world") != 11) { fprintf(stderr, "FAIL: string_length('hello world') != 11\n"); failures++; }

    FreeLibrary(hMod);

    if (failures == 0) {
        printf("All DLL tests passed!\n");
        return 0;
    } else {
        fprintf(stderr, "%d test(s) failed\n", failures);
        return 1;
    }
}
