#include <stdio.h>
#include <windows.h>

/* Separate function so it lives in .text and gets encrypted by the timer. */
#ifdef _MSC_VER
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

NOINLINE int compute(int a, int b) {
    int result = 0;
    for (int i = 0; i < a; i++) {
        result += b;
    }
    return result;
}

NOINLINE int validate(int x, int expected) {
    return x == expected;
}

int main(void) {
    /* Phase 1: compute before the encrypt timer fires. */
    int r1 = compute(6, 7);
    if (!validate(r1, 42)) {
        printf("FAIL: pre-sleep compute = %d (expected 42)\n", r1);
        return 1;
    }

    /* Sleep long enough for the fluctuation timer to encrypt .text.
       The test matrix uses --fluctuation-delay 50, so 100ms is plenty. */
    Sleep(100);

    /* Phase 2: compute AFTER .text has been encrypted.
       This forces the VEH to decrypt, proving the full cycle works. */
    int r2 = compute(8, 5);
    if (!validate(r2, 40)) {
        printf("FAIL: post-sleep compute = %d (expected 40)\n", r2);
        return 1;
    }

    /* Phase 3: sleep again and compute once more to test re-encryption. */
    Sleep(100);

    int r3 = compute(10, 3);
    if (!validate(r3, 30)) {
        printf("FAIL: second post-sleep compute = %d (expected 30)\n", r3);
        return 1;
    }

    printf("PASS: r1=%d r2=%d r3=%d\n", r1, r2, r3);
    return 0;
}
