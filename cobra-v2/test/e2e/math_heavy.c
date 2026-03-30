#include <stdio.h>
#include <math.h>

double newton_sqrt(double x) {
    if (x < 0) return -1;
    double guess = x / 2.0;
    for (int i = 0; i < 20; i++)
        guess = (guess + x / guess) / 2.0;
    return guess;
}

int gcd(int a, int b) {
    while (b != 0) {
        int t = b;
        b = a % b;
        a = t;
    }
    return a;
}

long factorial(int n) {
    long r = 1;
    for (int i = 2; i <= n; i++)
        r *= i;
    return r;
}

int is_prime(int n) {
    if (n < 2) return 0;
    if (n < 4) return 1;
    if (n % 2 == 0 || n % 3 == 0) return 0;
    for (int i = 5; i * i <= n; i += 6)
        if (n % i == 0 || n % (i + 2) == 0) return 0;
    return 1;
}

int main() {
    printf("sqrt(144)=%.0f sqrt(2)=%.4f\n", newton_sqrt(144), newton_sqrt(2));
    printf("gcd(48,18)=%d gcd(100,75)=%d\n", gcd(48, 18), gcd(100, 75));
    printf("fact(10)=%ld fact(12)=%ld\n", factorial(10), factorial(12));
    printf("prime: 2=%d 17=%d 100=%d 997=%d\n",
           is_prime(2), is_prime(17), is_prime(100), is_prime(997));
    return 0;
}
