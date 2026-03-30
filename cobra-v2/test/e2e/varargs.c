#include <stdio.h>
#include <stdarg.h>

int sum_va(int count, ...) {
    va_list args;
    va_start(args, count);
    int s = 0;
    for (int i = 0; i < count; i++)
        s += va_arg(args, int);
    va_end(args);
    return s;
}

int max_va(int count, ...) {
    va_list args;
    va_start(args, count);
    int m = va_arg(args, int);
    for (int i = 1; i < count; i++) {
        int v = va_arg(args, int);
        if (v > m) m = v;
    }
    va_end(args);
    return m;
}

int main() {
    printf("sum=%d max=%d\n", sum_va(5, 1, 2, 3, 4, 5), max_va(4, 3, 7, 2, 9));
    return 0;
}
