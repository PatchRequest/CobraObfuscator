#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Many branches — good CFF target
int complex_function(int n) {
    int result = 0;
    if (n > 100) result += 50; else result += 10;
    if (n > 50) result *= 2; else result *= 3;
    if (n > 25) result -= 5; else result -= 10;
    if (n > 75) result += n; else result += n / 2;
    if (n > 10) result ^= 0xFF; else result ^= 0xAA;
    if (n > 200) result = -result; else result = result + 1;
    if (n > 0) result += 42; else result -= 42;
    if (n % 2 == 0) result *= 2; else result *= 3;
    return result;
}

// Loop-heavy function
int loop_sum(int n) {
    int sum = 0;
    for (int i = 1; i <= n; i++) {
        if (i % 3 == 0) sum += i * 2;
        else if (i % 3 == 1) sum += i;
        else sum -= i / 2;
    }
    return sum;
}

// Nested loops
int matrix_trace(void) {
    int mat[4][4];
    int val = 1;
    for (int i = 0; i < 4; i++)
        for (int j = 0; j < 4; j++)
            mat[i][j] = val++;
    int trace = 0;
    for (int i = 0; i < 4; i++)
        trace += mat[i][i];
    return trace; // 1+6+11+16 = 34
}

// Pointer arithmetic
int array_search(const int *arr, int n, int target) {
    for (int i = 0; i < n; i++) {
        if (arr[i] == target) return i;
    }
    return -1;
}

// Heap allocation + string ops
unsigned int string_checksum(const char *s) {
    unsigned int h = 0;
    while (*s) {
        h = h * 31 + (unsigned char)*s;
        s++;
    }
    return h;
}

int main(void) {
    int pass = 0, fail = 0;

#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Medium Test Suite ===\n");

    CHECK(complex_function(150) == 106, "complex(150)");
    CHECK(complex_function(30) == 516, "complex(30)");
    CHECK(complex_function(0) == 298, "complex(0)");

    CHECK(loop_sum(10) == 51, "loop_sum(10)");
    CHECK(loop_sum(100) == 4266, "loop_sum(100)");

    CHECK(matrix_trace() == 34, "matrix_trace");

    int arr[] = {10, 20, 30, 40, 50};
    CHECK(array_search(arr, 5, 30) == 2, "array_search found");
    CHECK(array_search(arr, 5, 99) == -1, "array_search not found");

    CHECK(string_checksum("hello") == 99162322, "string_checksum('hello')");
    CHECK(string_checksum("") == 0, "string_checksum('')");

    // Heap string
    char *heap = (char *)malloc(64);
    strcpy(heap, "obfuscation-test");
    CHECK(strcmp(heap, "obfuscation-test") == 0, "heap string intact");
    unsigned int h1 = string_checksum(heap);
    unsigned int h2 = string_checksum("obfuscation-test");
    CHECK(h1 == h2, "heap string hash matches literal");
    free(heap);

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
