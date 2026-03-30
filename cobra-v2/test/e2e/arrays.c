#include <stdio.h>

void bubble_sort(int *arr, int n) {
    for (int i = 0; i < n - 1; i++)
        for (int j = 0; j < n - i - 1; j++)
            if (arr[j] > arr[j + 1]) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
}

int binary_search(int *arr, int n, int target) {
    int lo = 0, hi = n - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (arr[mid] == target) return mid;
        else if (arr[mid] < target) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

void matrix_mul(int a[2][2], int b[2][2], int c[2][2]) {
    for (int i = 0; i < 2; i++)
        for (int j = 0; j < 2; j++) {
            c[i][j] = 0;
            for (int k = 0; k < 2; k++)
                c[i][j] += a[i][k] * b[k][j];
        }
}

int main() {
    int arr[] = {5, 3, 8, 1, 9, 2, 7, 4, 6};
    bubble_sort(arr, 9);
    printf("sorted=");
    for (int i = 0; i < 9; i++) printf("%d", arr[i]);
    printf("\n");

    printf("find7=%d find10=%d\n", binary_search(arr, 9, 7), binary_search(arr, 9, 10));

    int a[2][2] = {{1, 2}, {3, 4}};
    int b[2][2] = {{5, 6}, {7, 8}};
    int c[2][2];
    matrix_mul(a, b, c);
    printf("mat=[%d,%d;%d,%d]\n", c[0][0], c[0][1], c[1][0], c[1][1]);
    return 0;
}
