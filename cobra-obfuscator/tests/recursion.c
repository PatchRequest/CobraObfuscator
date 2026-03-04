// Tests: mutual recursion, deep recursion, tail-call patterns
#include <stdio.h>

static int is_even(int n);
static int is_odd(int n);

// Mutual recursion
static int is_even(int n) {
    if (n == 0) return 1;
    return is_odd(n - 1);
}

static int is_odd(int n) {
    if (n == 0) return 0;
    return is_even(n - 1);
}

// Ackermann (deep recursion, many stack frames)
static int ackermann(int m, int n) {
    if (m == 0) return n + 1;
    if (n == 0) return ackermann(m - 1, 1);
    return ackermann(m - 1, ackermann(m, n - 1));
}

// Power via recursion
static long long power(int base, int exp) {
    if (exp == 0) return 1;
    if (exp % 2 == 0) {
        long long half = power(base, exp / 2);
        return half * half;
    }
    return base * power(base, exp - 1);
}

// GCD
static int gcd(int a, int b) {
    if (b == 0) return a;
    return gcd(b, a % b);
}

// Tower of Hanoi — count moves
static int hanoi_count;
static void hanoi(int n, int from, int to, int aux) {
    if (n == 0) return;
    hanoi(n - 1, from, aux, to);
    hanoi_count++;
    hanoi(n - 1, aux, to, from);
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Recursion Tests ===\n");

    CHECK(is_even(10) == 1, "is_even(10)");
    CHECK(is_even(7) == 0, "is_even(7)");
    CHECK(is_odd(11) == 1, "is_odd(11)");
    CHECK(is_odd(4) == 0, "is_odd(4)");

    CHECK(ackermann(0, 0) == 1, "ack(0,0)");
    CHECK(ackermann(1, 1) == 3, "ack(1,1)");
    CHECK(ackermann(2, 2) == 7, "ack(2,2)");
    CHECK(ackermann(3, 3) == 61, "ack(3,3)");

    CHECK(power(2, 10) == 1024, "power(2,10)");
    CHECK(power(3, 7) == 2187, "power(3,7)");
    CHECK(power(5, 0) == 1, "power(5,0)");

    CHECK(gcd(48, 18) == 6, "gcd(48,18)");
    CHECK(gcd(100, 75) == 25, "gcd(100,75)");
    CHECK(gcd(17, 13) == 1, "gcd(17,13)");

    hanoi_count = 0;
    hanoi(10, 1, 3, 2);
    CHECK(hanoi_count == 1023, "hanoi(10) = 1023 moves");

    hanoi_count = 0;
    hanoi(15, 1, 3, 2);
    CHECK(hanoi_count == 32767, "hanoi(15) = 32767 moves");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
