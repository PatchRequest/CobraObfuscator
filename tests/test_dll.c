#include <windows.h>
#include <stdint.h>

// Exported functions that exercise various code patterns

__declspec(dllexport) int add_numbers(int a, int b) {
    return a + b;
}

__declspec(dllexport) int fibonacci(int n) {
    if (n <= 1) return n;
    int a = 0, b = 1;
    for (int i = 2; i <= n; i++) {
        int tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}

__declspec(dllexport) uint32_t xor_hash(const char *data, int len) {
    uint32_t hash = 0x811c9dc5;
    for (int i = 0; i < len; i++) {
        hash ^= (uint8_t)data[i];
        hash *= 0x01000193;
    }
    return hash;
}

__declspec(dllexport) void bubble_sort(int *arr, int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
        }
    }
}

__declspec(dllexport) int matrix_trace(int matrix[4][4]) {
    int trace = 0;
    for (int i = 0; i < 4; i++) {
        trace += matrix[i][i];
    }
    return trace;
}

__declspec(dllexport) int string_length(const char *s) {
    int len = 0;
    while (s[len] != '\0') len++;
    return len;
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpReserved) {
    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        case DLL_PROCESS_DETACH:
            break;
    }
    return TRUE;
}
