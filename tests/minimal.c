#include <stdio.h>

int compute(int a, int b) {
    int result = 0;
    if (a > b) {
        result = a - b;
    } else {
        result = b - a;
    }
    result = result * 2;
    return result;
}

int main(void) {
    int x = compute(10, 3);
    int y = compute(3, 10);
    if (x == 14 && y == 14) {
        printf("PASS: x=%d y=%d\n", x, y);
        return 0;
    } else {
        printf("FAIL: x=%d y=%d\n", x, y);
        return 1;
    }
}
