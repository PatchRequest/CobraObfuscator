#include <stdio.h>

int sum_for(int n) {
    int s = 0;
    for (int i = 1; i <= n; i++)
        s += i;
    return s;
}

int sum_while(int n) {
    int s = 0, i = 1;
    while (i <= n) {
        s += i;
        i++;
    }
    return s;
}

int sum_dowhile(int n) {
    int s = 0, i = 1;
    do {
        s += i;
        i++;
    } while (i <= n);
    return s;
}

int nested_sum(int rows, int cols) {
    int s = 0;
    for (int i = 0; i < rows; i++)
        for (int j = 0; j < cols; j++)
            s += i * cols + j;
    return s;
}

int main() {
    printf("for=%d while=%d dowhile=%d nested=%d\n",
           sum_for(10), sum_while(10), sum_dowhile(10), nested_sum(3, 4));
    return 0;
}
