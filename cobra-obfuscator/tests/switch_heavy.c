// Tests: large switch statements, jump tables, fall-through
#include <stdio.h>

// Large switch — compilers often use jump tables
int big_switch(int x) {
    switch (x) {
        case 0:  return 100;
        case 1:  return 201;
        case 2:  return 304;
        case 3:  return 409;
        case 4:  return 516;
        case 5:  return 625;
        case 6:  return 736;
        case 7:  return 849;
        case 8:  return 964;
        case 9:  return 1081;
        case 10: return 1200;
        case 11: return 1321;
        case 12: return 1444;
        case 13: return 1569;
        case 14: return 1696;
        case 15: return 1825;
        default: return -1;
    }
}

// Sparse switch — compiler may use if-else chain or binary search
int sparse_switch(int x) {
    switch (x) {
        case 1:    return 10;
        case 10:   return 20;
        case 100:  return 30;
        case 1000: return 40;
        case 5000: return 50;
        case 9999: return 60;
        default:   return 0;
    }
}

// Nested switch
int nested_switch(int a, int b) {
    switch (a) {
        case 0:
            switch (b) {
                case 0: return 1;
                case 1: return 2;
                default: return 3;
            }
        case 1:
            switch (b) {
                case 0: return 10;
                case 1: return 20;
                default: return 30;
            }
        default:
            return a * 100 + b;
    }
}

// State machine via switch
int state_machine(const char *input) {
    int state = 0;
    int count = 0;
    for (int i = 0; input[i]; i++) {
        char c = input[i];
        switch (state) {
            case 0: // start
                if (c == 'a') state = 1;
                else state = 0;
                break;
            case 1: // saw 'a'
                if (c == 'b') state = 2;
                else if (c == 'a') state = 1;
                else state = 0;
                break;
            case 2: // saw 'ab'
                if (c == 'c') { count++; state = 0; }
                else if (c == 'a') state = 1;
                else state = 0;
                break;
        }
    }
    return count; // count occurrences of "abc"
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Switch Tests ===\n");

    // Big switch
    CHECK(big_switch(0) == 100, "big_switch(0)");
    CHECK(big_switch(7) == 849, "big_switch(7)");
    CHECK(big_switch(15) == 1825, "big_switch(15)");
    CHECK(big_switch(16) == -1, "big_switch(16) default");
    int bsum = 0;
    for (int i = 0; i <= 15; i++) bsum += big_switch(i);
    CHECK(bsum == 14840, "big_switch sum 0..15");

    // Sparse switch
    CHECK(sparse_switch(1) == 10, "sparse(1)");
    CHECK(sparse_switch(100) == 30, "sparse(100)");
    CHECK(sparse_switch(9999) == 60, "sparse(9999)");
    CHECK(sparse_switch(42) == 0, "sparse(42) default");

    // Nested switch
    CHECK(nested_switch(0, 0) == 1, "nested(0,0)");
    CHECK(nested_switch(0, 1) == 2, "nested(0,1)");
    CHECK(nested_switch(1, 0) == 10, "nested(1,0)");
    CHECK(nested_switch(1, 1) == 20, "nested(1,1)");
    CHECK(nested_switch(3, 5) == 305, "nested(3,5) default");

    // State machine
    CHECK(state_machine("abc") == 1, "sm 'abc'");
    CHECK(state_machine("abcabc") == 2, "sm 'abcabc'");
    CHECK(state_machine("aabcxabc") == 2, "sm 'aabcxabc'");
    CHECK(state_machine("xyz") == 0, "sm 'xyz'");
    CHECK(state_machine("ababc") == 1, "sm 'ababc'");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
