// Tests: deep nesting, while/do-while/for, break/continue, goto
#include <stdio.h>
#include <stdlib.h>

int nested_loops(int n) {
    int count = 0;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < n; k++) {
                if ((i + j + k) % 7 == 0)
                    count++;
            }
        }
    }
    return count;
}

int while_with_break(int *arr, int n) {
    int i = 0;
    int sum = 0;
    while (i < n) {
        if (arr[i] < 0) break;
        sum += arr[i];
        i++;
    }
    return sum;
}

int do_while_test(int start) {
    int val = start;
    int iters = 0;
    do {
        val = (val * 3 + 1) % 1000;
        iters++;
    } while (val != start && iters < 10000);
    return iters;
}

int continue_test(int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        if (i % 3 == 0) continue;
        if (i % 5 == 0) continue;
        sum += i;
    }
    return sum;
}

int goto_test(int n) {
    int result = 0;
    int i = 0;
loop:
    if (i >= n) goto done;
    result += i * i;
    i++;
    goto loop;
done:
    return result;
}

// Bubble sort — lots of swaps and branches
void bubble_sort(int *arr, int n) {
    for (int i = 0; i < n - 1; i++) {
        int swapped = 0;
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
                swapped = 1;
            }
        }
        if (!swapped) break;
    }
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Loop Tests ===\n");

    CHECK(nested_loops(10) == 140, "nested_loops(10)");
    CHECK(nested_loops(5) == 19, "nested_loops(5)");

    int arr1[] = {1, 2, 3, -1, 5};
    CHECK(while_with_break(arr1, 5) == 6, "while_with_break stops at -1");
    int arr2[] = {10, 20, 30};
    CHECK(while_with_break(arr2, 3) == 60, "while_with_break no break");

    CHECK(do_while_test(1) == 100, "do_while collatz-like(1)");

    CHECK(continue_test(20) == 112, "continue skips %3 and %5");

    CHECK(goto_test(10) == 285, "goto sum of squares");
    CHECK(goto_test(0) == 0, "goto n=0");

    // Bubble sort
    int data[] = {5, 3, 8, 1, 9, 2, 7, 4, 6, 0};
    bubble_sort(data, 10);
    int sorted = 1;
    for (int i = 0; i < 10; i++) {
        if (data[i] != i) { sorted = 0; break; }
    }
    CHECK(sorted, "bubble_sort");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
