#include <stdio.h>

unsigned int popcount(unsigned int x) {
    unsigned int count = 0;
    while (x) {
        count += x & 1;
        x >>= 1;
    }
    return count;
}

unsigned int reverse_bits(unsigned int x) {
    unsigned int r = 0;
    for (int i = 0; i < 32; i++) {
        r = (r << 1) | (x & 1);
        x >>= 1;
    }
    return r;
}

int is_power_of_two(unsigned int x) {
    return x != 0 && (x & (x - 1)) == 0;
}

unsigned int next_power_of_two(unsigned int x) {
    x--;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x++;
    return x;
}

int main() {
    printf("pop(0xFF)=%u pop(0x1234)=%u\n", popcount(0xFF), popcount(0x1234));
    printf("rev(1)=%u rev(0x80000000)=%u\n", reverse_bits(1), reverse_bits(0x80000000u));
    printf("pow2: 1=%d 4=%d 6=%d 128=%d\n",
           is_power_of_two(1), is_power_of_two(4),
           is_power_of_two(6), is_power_of_two(128));
    printf("next: 3->%u 5->%u 17->%u\n",
           next_power_of_two(3), next_power_of_two(5), next_power_of_two(17));
    return 0;
}
