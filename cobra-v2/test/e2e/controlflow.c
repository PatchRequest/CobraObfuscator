#include <stdio.h>
int classify(int n) {
    if (n > 0) return 1;
    else if (n < 0) return -1;
    else return 0;
}
int main() {
    printf("pos=%d neg=%d zero=%d\n", classify(5), classify(-3), classify(0));
    return 0;
}
