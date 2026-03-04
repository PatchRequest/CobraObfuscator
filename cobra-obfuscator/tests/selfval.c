#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

// ========== TLS Callback ==========
static volatile LONG g_tls_count = 0;

void NTAPI tls_callback(PVOID DllHandle, DWORD dwReason, PVOID Reserved) {
    (void)DllHandle; (void)Reserved;
    if (dwReason == DLL_PROCESS_ATTACH || dwReason == DLL_THREAD_ATTACH) {
        InterlockedIncrement(&g_tls_count);
    }
}

#ifdef _MSC_VER
#pragma section(".CRT$XLB", read)
__declspec(allocate(".CRT$XLB")) PIMAGE_TLS_CALLBACK _tls_cb = tls_callback;
#else
__attribute__((section(".CRT$XLB"))) PIMAGE_TLS_CALLBACK _tls_cb = tls_callback;
#endif

// ========== Recursion ==========
static int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

static long long factorial(int n) {
    if (n <= 1) return 1;
    return (long long)n * factorial(n - 1);
}

static int recursive_sum(int n) {
    if (n <= 0) return 0;
    return n + recursive_sum(n - 1);
}

// ========== Switch/Dispatch ==========
typedef enum { OP_ADD, OP_SUB, OP_MUL, OP_XOR, OP_ROL } Op;

static unsigned int dispatch_op(Op op, unsigned int a, unsigned int b) {
    switch (op) {
        case OP_ADD: return a + b;
        case OP_SUB: return a - b;
        case OP_MUL: return a * b;
        case OP_XOR: return a ^ b;
        case OP_ROL: return (a << b) | (a >> (32 - b));
        default: return 0;
    }
}

// ========== Heap linked list ==========
typedef struct Node { int value; struct Node *next; } Node;

static int linked_list_sum(int count) {
    Node *head = NULL;
    for (int i = 1; i <= count; i++) {
        Node *n = (Node *)malloc(sizeof(Node));
        n->value = i;
        n->next = head;
        head = n;
    }
    int sum = 0;
    Node *cur = head;
    while (cur) {
        sum += cur->value;
        Node *tmp = cur;
        cur = cur->next;
        free(tmp);
    }
    return sum;
}

// ========== LCG + sort ==========
static unsigned int lcg_next(unsigned int *state) {
    *state = *state * 1103515245 + 12345;
    return (*state >> 16) & 0x7fff;
}

static void selection_sort(unsigned int *arr, int n) {
    for (int i = 0; i < n - 1; i++) {
        int min_idx = i;
        for (int j = i + 1; j < n; j++) {
            if (arr[j] < arr[min_idx]) min_idx = j;
        }
        if (min_idx != i) {
            unsigned int tmp = arr[i];
            arr[i] = arr[min_idx];
            arr[min_idx] = tmp;
        }
    }
}

// ========== Stack checksum ==========
typedef struct {
    unsigned int data[16];
    unsigned int checksum;
} CheckBlock;

static void init_check_block(CheckBlock *blk, unsigned int seed) {
    unsigned int state = seed;
    unsigned int sum = 0;
    for (int i = 0; i < 16; i++) {
        blk->data[i] = lcg_next(&state);
        sum += blk->data[i];
    }
    blk->checksum = sum;
}

static int verify_check_block(const CheckBlock *blk) {
    unsigned int sum = 0;
    for (int i = 0; i < 16; i++) sum += blk->data[i];
    return sum == blk->checksum;
}

// ========== String hash ==========
static unsigned int djb2_hash(const char *str) {
    unsigned int hash = 5381;
    int c;
    while ((c = *str++)) {
        hash = ((hash << 5) + hash) + (unsigned int)c;
    }
    return hash;
}

// ========== Threading ==========
typedef struct { int id; int result; } ThreadArg;

static DWORD WINAPI thread_func(LPVOID param) {
    ThreadArg *arg = (ThreadArg *)param;
    arg->result = fibonacci(10 + arg->id);
    return 0;
}

static volatile LONG g_cs_counter = 0;
static CRITICAL_SECTION g_cs;

static DWORD WINAPI cs_thread_func(LPVOID param) {
    (void)param;
    for (int i = 0; i < 1000; i++) {
        EnterCriticalSection(&g_cs);
        g_cs_counter++;
        LeaveCriticalSection(&g_cs);
    }
    return 0;
}

// ========== Main ==========
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [ OK ] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

int main(void) {
    int pass = 0, fail = 0;

    printf("========================================\n");
    printf("  Self-Validating Test Executable\n");
    printf("========================================\n\n");

    // 1. TLS
    printf("[1] TLS callback tests\n");
    CHECK(g_tls_count >= 1, "TLS callback executed before main");

    // 2. Recursion
    printf("\n[2] Recursion tests\n");
    CHECK(fibonacci(10) == 55, "fibonacci(10) == 55");
    CHECK(fibonacci(20) == 6765, "fibonacci(20) == 6765");
    CHECK(factorial(10) == 3628800, "factorial(10) == 3628800");
    CHECK(recursive_sum(10) == 55, "recursive_sum(1..10) == 55");

    // 3. Switch dispatch
    printf("\n[3] Switch/dispatch tests\n");
    CHECK(dispatch_op(OP_ADD, 10, 20) == 30, "ADD(10, 20) == 30");
    CHECK(dispatch_op(OP_SUB, 50, 17) == 33, "SUB(50, 17) == 33");
    CHECK(dispatch_op(OP_MUL, 6, 7) == 42,   "MUL(6, 7) == 42");
    CHECK(dispatch_op(OP_XOR, 0xFF, 0x0F) == 0xF0, "XOR(0xFF, 0x0F) == 0xF0");
    CHECK(dispatch_op(OP_ROL, 1, 4) == 16,   "ROL(1, 4) == 16");

    // Op chain
    unsigned int v = 100;
    v = dispatch_op(OP_ADD, v, 3);
    v = dispatch_op(OP_SUB, v, 3);
    v = dispatch_op(OP_MUL, v, 3);
    v = dispatch_op(OP_XOR, v, 3);
    v = dispatch_op(OP_ROL, v, 3);
    CHECK(v == 2424, "op chain final == 2424");

    // 4. Heap linked list
    printf("\n[4] Heap linked-list tests\n");
    CHECK(linked_list_sum(100) == 5050, "linked list sum(1..100) == 5050");

    // 5. LCG + sort
    printf("\n[5] Heap array + LCG tests\n");
    {
        #define MAGIC_SEED 0xDEADBEEF
        #define ARR_SIZE 64
        unsigned int *arr = (unsigned int *)malloc(ARR_SIZE * sizeof(unsigned int));
        unsigned int state = MAGIC_SEED;
        arr[0] = MAGIC_SEED;
        for (int i = 1; i < ARR_SIZE; i++) arr[i] = lcg_next(&state);

        unsigned int state2 = MAGIC_SEED;
        int det = 1;
        for (int i = 1; i < ARR_SIZE; i++) {
            unsigned int v2 = lcg_next(&state2);
            if (arr[i] != v2) { det = 0; break; }
        }
        CHECK(det, "LCG sequence deterministic");
        CHECK(arr[0] == MAGIC_SEED, "arr[0] == MAGIC_SEED");

        unsigned long long sum_before = 0;
        for (int i = 0; i < ARR_SIZE; i++) sum_before += arr[i];
        selection_sort(arr, ARR_SIZE);
        int sorted = 1;
        for (int i = 1; i < ARR_SIZE; i++) {
            if (arr[i] < arr[i-1]) { sorted = 0; break; }
        }
        CHECK(sorted, "selection sort produces sorted array");
        unsigned long long sum_after = 0;
        for (int i = 0; i < ARR_SIZE; i++) sum_after += arr[i];
        CHECK(sum_before == sum_after, "sum preserved after sort");
        free(arr);
    }

    // 6. Stack checksum
    printf("\n[6] Stack struct checksum tests\n");
    {
        CheckBlock b1, b2, b3;
        init_check_block(&b1, 42);
        init_check_block(&b2, 0xCAFEBABE);
        init_check_block(&b3, 0);
        CHECK(verify_check_block(&b1), "stack block (seed=42) valid");
        CHECK(verify_check_block(&b2), "stack block (seed=0xCAFEBABE) valid");
        CHECK(verify_check_block(&b3), "stack block (seed=0) valid");
    }

    // 7. String hash
    printf("\n[7] String tests\n");
    {
        unsigned int h1 = djb2_hash("hello");
        unsigned int h2 = djb2_hash("hello");
        unsigned int h3 = djb2_hash("world");
        CHECK(h1 == h2, "hash deterministic");
        CHECK(h1 != h3, "hash differs for different strings");
        CHECK(h1 == 0x0F923099, "hash('hello') == 0x0F923099");

        char *heap_str = (char *)malloc(32);
        strcpy(heap_str, "self-validation-passed");
        CHECK(strcmp(heap_str, "self-validation-passed") == 0, "heap string == 'self-validation-passed'");
        unsigned int hh1 = djb2_hash(heap_str);
        unsigned int hh2 = djb2_hash("self-validation-passed");
        CHECK(hh1 == hh2, "heap string hash consistent");
        free(heap_str);
    }

    // 8. Deep recursion
    printf("\n[8] Deep recursion stress\n");
    CHECK(fibonacci(25) == 75025, "fibonacci(25) == 75025");

    // 9. Windows API
    printf("\n[9] Windows API tests\n");
    {
        HMODULE k32 = GetModuleHandleA("kernel32.dll");
        FARPROC gpa = GetProcAddress(k32, "GetCurrentProcessId");
        CHECK(k32 != NULL && gpa != NULL, "GetModuleHandle + GetProcAddress");

        LPVOID mem = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        MEMORY_BASIC_INFORMATION mbi;
        VirtualQuery(mem, &mbi, sizeof(mbi));
        CHECK(mem != NULL && mbi.RegionSize >= 4096, "VirtualAlloc/Query/Free");
        VirtualFree(mem, 0, MEM_RELEASE);

        HANDLE heap = GetProcessHeap();
        LPVOID hm = HeapAlloc(heap, 0, 1024);
        CHECK(hm != NULL, "HeapAlloc/HeapFree");
        HeapFree(heap, 0, hm);

        char path[MAX_PATH];
        GetModuleFileNameA(NULL, path, MAX_PATH);
        CHECK(strlen(path) > 0, "GetModuleFileName");

        char envbuf[256];
        DWORD r = GetEnvironmentVariableA("PATH", envbuf, sizeof(envbuf));
        CHECK(r > 0, "GetEnvironmentVariable");
    }

    // 10. Multi-threaded
    printf("\n[10] Multi-threaded tests\n");
    {
        #define NUM_THREADS 4
        ThreadArg args[NUM_THREADS];
        HANDLE threads[NUM_THREADS];
        int expected[] = { 55, 89, 144, 233 }; // fib(10..13)

        for (int i = 0; i < NUM_THREADS; i++) {
            args[i].id = i;
            args[i].result = 0;
            threads[i] = CreateThread(NULL, 0, thread_func, &args[i], 0, NULL);
        }
        WaitForMultipleObjects(NUM_THREADS, threads, TRUE, INFINITE);

        int all_correct = 1;
        for (int i = 0; i < NUM_THREADS; i++) {
            if (args[i].result != expected[i]) all_correct = 0;
            CloseHandle(threads[i]);
        }
        CHECK(all_correct, "4 threads complete with correct results");

        // Critical section test
        InitializeCriticalSection(&g_cs);
        g_cs_counter = 0;
        HANDLE cs_threads[4];
        for (int i = 0; i < 4; i++) {
            cs_threads[i] = CreateThread(NULL, 0, cs_thread_func, NULL, 0, NULL);
        }
        WaitForMultipleObjects(4, cs_threads, TRUE, INFINITE);
        for (int i = 0; i < 4; i++) CloseHandle(cs_threads[i]);
        CHECK(g_cs_counter == 4000, "critical section counter");
        DeleteCriticalSection(&g_cs);

        CHECK(g_tls_count >= 1 + NUM_THREADS, "TLS callback count >= 1+threads");
    }

    printf("\n========================================\n");
    if (fail == 0) {
        printf("  ALL %d CHECKS PASSED\n", pass);
    } else {
        printf("  %d PASSED, %d FAILED\n", pass, fail);
    }
    printf("========================================\n");

    return fail > 0 ? 1 : 0;
}
