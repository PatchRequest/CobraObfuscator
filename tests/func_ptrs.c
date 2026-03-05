// Tests: function pointers, callbacks, indirect calls, vtable-like dispatch
// Stresses CFG analysis — the obfuscator must handle indirect call targets
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Basic function pointer dispatch
typedef int (*BinOp)(int, int);

static int add(int a, int b) { return a + b; }
static int sub(int a, int b) { return a - b; }
static int mul(int a, int b) { return a * b; }
static int my_div(int a, int b) { return b != 0 ? a / b : 0; }
static int my_mod(int a, int b) { return b != 0 ? a % b : 0; }

static int apply(BinOp fn, int a, int b) { return fn(a, b); }

// Callback-based iteration
typedef void (*Visitor)(int index, int value, void *ctx);

static void for_each(const int *arr, int n, Visitor visit, void *ctx) {
    for (int i = 0; i < n; i++)
        visit(i, arr[i], ctx);
}

static void sum_visitor(int index, int value, void *ctx) {
    (void)index;
    *(int *)ctx += value;
}

static void max_visitor(int index, int value, void *ctx) {
    (void)index;
    int *max = (int *)ctx;
    if (value > *max) *max = value;
}

// qsort comparators
static int cmp_asc(const void *a, const void *b) {
    return *(const int *)a - *(const int *)b;
}

static int cmp_desc(const void *a, const void *b) {
    return *(const int *)b - *(const int *)a;
}

// Vtable-like dispatch
typedef struct Shape Shape;
struct ShapeVtable {
    int (*area)(const Shape *);
    int (*perimeter)(const Shape *);
    const char *name;
};
struct Shape {
    const struct ShapeVtable *vt;
    int w, h;
};

static int rect_area(const Shape *s) { return s->w * s->h; }
static int rect_perimeter(const Shape *s) { return 2 * (s->w + s->h); }
static const struct ShapeVtable rect_vt = { rect_area, rect_perimeter, "rectangle" };

static int square_area(const Shape *s) { return s->w * s->w; }
static int square_perimeter(const Shape *s) { return 4 * s->w; }
static const struct ShapeVtable square_vt = { square_area, square_perimeter, "square" };

static int triangle_area(const Shape *s) { return (s->w * s->h) / 2; }
static int triangle_perimeter(const Shape *s) { return s->w + s->h + s->w; } // approximate
static const struct ShapeVtable triangle_vt = { triangle_area, triangle_perimeter, "triangle" };

// Function pointer array as state machine
typedef int (*StateFunc)(int input, int *next_state);

static int state_idle(int input, int *next_state) {
    if (input == 1) { *next_state = 1; return 10; }
    *next_state = 0;
    return 0;
}

static int state_running(int input, int *next_state) {
    if (input == 2) { *next_state = 2; return 20; }
    if (input == 0) { *next_state = 0; return 5; }
    *next_state = 1;
    return 15;
}

static int state_done(int input, int *next_state) {
    (void)input;
    *next_state = 0;
    return 100;
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Function Pointer Tests ===\n");

    // Basic dispatch
    CHECK(apply(add, 10, 20) == 30, "apply add");
    CHECK(apply(sub, 50, 17) == 33, "apply sub");
    CHECK(apply(mul, 6, 7) == 42, "apply mul");
    CHECK(apply(my_div, 100, 3) == 33, "apply div");
    CHECK(apply(my_mod, 100, 3) == 1, "apply mod");

    // Array of function pointers
    BinOp ops[] = { add, sub, mul, my_div, my_mod };
    int results[] = { 30, -10, 200, 2, 0 };
    int all_ok = 1;
    for (int i = 0; i < 5; i++) {
        if (ops[i](10, 20) != results[i]) all_ok = 0;
    }
    CHECK(all_ok, "op array dispatch");

    // Callback iteration
    int arr[] = {3, 7, 1, 9, 2, 8, 4, 6, 5, 10};
    int sum = 0;
    for_each(arr, 10, sum_visitor, &sum);
    CHECK(sum == 55, "for_each sum");

    int mx = arr[0];
    for_each(arr, 10, max_visitor, &mx);
    CHECK(mx == 10, "for_each max");

    // qsort with function pointers
    int sortme[] = {5, 3, 8, 1, 9, 2, 7, 4, 6, 0};
    qsort(sortme, 10, sizeof(int), cmp_asc);
    int sorted_asc = 1;
    for (int i = 0; i < 10; i++) if (sortme[i] != i) sorted_asc = 0;
    CHECK(sorted_asc, "qsort ascending");

    qsort(sortme, 10, sizeof(int), cmp_desc);
    int sorted_desc = 1;
    for (int i = 0; i < 10; i++) if (sortme[i] != 9 - i) sorted_desc = 0;
    CHECK(sorted_desc, "qsort descending");

    // Vtable dispatch
    Shape rect = { &rect_vt, 5, 3 };
    Shape sq = { &square_vt, 4, 0 };
    Shape tri = { &triangle_vt, 6, 4 };

    CHECK(rect.vt->area(&rect) == 15, "rect area");
    CHECK(rect.vt->perimeter(&rect) == 16, "rect perimeter");
    CHECK(sq.vt->area(&sq) == 16, "square area");
    CHECK(sq.vt->perimeter(&sq) == 16, "square perimeter");
    CHECK(tri.vt->area(&tri) == 12, "triangle area");
    CHECK(strcmp(rect.vt->name, "rectangle") == 0, "rect name");
    CHECK(strcmp(sq.vt->name, "square") == 0, "square name");

    // Polymorphic array
    Shape *shapes[] = { &rect, &sq, &tri };
    int total_area = 0;
    for (int i = 0; i < 3; i++)
        total_area += shapes[i]->vt->area(shapes[i]);
    CHECK(total_area == 43, "polymorphic total area");

    // State machine via function pointer array
    StateFunc states[] = { state_idle, state_running, state_done };
    int inputs[] = { 1, 3, 2, 0 };
    int state = 0;
    int total_output = 0;
    for (int i = 0; i < 4; i++) {
        total_output += states[state](inputs[i], &state);
    }
    // idle(1)->10, state=1; running(3)->15, state=1; running(2)->20, state=2; done(0)->100, state=0
    CHECK(total_output == 145, "state machine total output");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
