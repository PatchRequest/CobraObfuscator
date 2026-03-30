#include <cstdio>
#include <stdexcept>

int safe_div(int a, int b) {
    if (b == 0) throw std::runtime_error("div by zero");
    return a / b;
}

int safe_sqrt(int x) {
    if (x < 0) throw std::invalid_argument("negative");
    int r = 0;
    while ((r + 1) * (r + 1) <= x) r++;
    return r;
}

int main() {
    // Normal cases
    printf("div(10,3)=%d sqrt(16)=%d\n", safe_div(10, 3), safe_sqrt(16));

    // Exception cases
    try {
        safe_div(1, 0);
        printf("ERROR: no throw\n");
    } catch (const std::runtime_error &e) {
        printf("caught: %s\n", e.what());
    }

    try {
        safe_sqrt(-1);
        printf("ERROR: no throw\n");
    } catch (const std::invalid_argument &e) {
        printf("caught: %s\n", e.what());
    }

    // Nested try-catch
    int result = 0;
    try {
        result = safe_div(100, safe_div(10, 2));
    } catch (...) {
        result = -1;
    }
    printf("nested=%d\n", result);
    return 0;
}
