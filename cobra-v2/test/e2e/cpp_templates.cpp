#include <cstdio>

template<typename T>
T max_of(T a, T b) {
    return (a > b) ? a : b;
}

template<typename T>
T min_of(T a, T b) {
    return (a < b) ? a : b;
}

template<typename T, int N>
class StaticArray {
    T data[N];
    int count;
public:
    StaticArray() : count(0) {}
    void push(T val) { if (count < N) data[count++] = val; }
    T get(int i) const { return data[i]; }
    int size() const { return count; }

    T sum() const {
        T s = T();
        for (int i = 0; i < count; i++)
            s += data[i];
        return s;
    }

    T max() const {
        T m = data[0];
        for (int i = 1; i < count; i++)
            if (data[i] > m) m = data[i];
        return m;
    }
};

int main() {
    printf("max(3,7)=%d max(2.5,1.5)=%.1f\n", max_of(3, 7), max_of(2.5, 1.5));
    printf("min(3,7)=%d min(2.5,1.5)=%.1f\n", min_of(3, 7), min_of(2.5, 1.5));

    StaticArray<int, 10> arr;
    for (int i = 1; i <= 5; i++) arr.push(i * 10);
    printf("size=%d sum=%d max=%d\n", arr.size(), arr.sum(), arr.max());
    return 0;
}
