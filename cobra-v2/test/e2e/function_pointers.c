#include <stdio.h>

int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
int mul(int a, int b) { return a * b; }

typedef int (*binop)(int, int);

int apply(binop fn, int a, int b) {
    return fn(a, b);
}

int reduce(binop fn, int *arr, int n) {
    int result = arr[0];
    for (int i = 1; i < n; i++)
        result = fn(result, arr[i]);
    return result;
}

int main() {
    binop ops[] = {add, sub, mul};
    printf("apply: add=%d sub=%d mul=%d\n",
           apply(ops[0], 10, 3), apply(ops[1], 10, 3), apply(ops[2], 10, 3));

    int arr[] = {1, 2, 3, 4, 5};
    printf("reduce: sum=%d product=%d\n",
           reduce(add, arr, 5), reduce(mul, arr, 5));
    return 0;
}
