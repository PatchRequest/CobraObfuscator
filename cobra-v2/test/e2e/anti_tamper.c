#include <stdio.h>

int square(int x) {
    return x * x;
}

int cube(int x) {
    return x * x * x;
}

int main() {
    printf("sq=%d cube=%d\n", square(5), cube(3));
    return 0;
}
