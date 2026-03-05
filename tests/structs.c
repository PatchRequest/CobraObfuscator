// Tests: struct passing, unions, nested structs, memcpy patterns
// Stresses stack layout and register allocation after obfuscation
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

typedef struct { float x, y; } Vec2;
typedef struct { float x, y, z; } Vec3;
typedef struct { float m[3][3]; } Mat3;

static Vec2 vec2_add(Vec2 a, Vec2 b) {
    Vec2 r = { a.x + b.x, a.y + b.y };
    return r;
}

static float vec2_dot(Vec2 a, Vec2 b) {
    return a.x * b.x + a.y * b.y;
}

static Vec3 vec3_cross(Vec3 a, Vec3 b) {
    Vec3 r;
    r.x = a.y * b.z - a.z * b.y;
    r.y = a.z * b.x - a.x * b.z;
    r.z = a.x * b.y - a.y * b.x;
    return r;
}

static Mat3 mat3_identity(void) {
    Mat3 m;
    memset(&m, 0, sizeof(m));
    m.m[0][0] = 1.0f; m.m[1][1] = 1.0f; m.m[2][2] = 1.0f;
    return m;
}

static Mat3 mat3_mul(Mat3 a, Mat3 b) {
    Mat3 r;
    for (int i = 0; i < 3; i++)
        for (int j = 0; j < 3; j++) {
            r.m[i][j] = 0;
            for (int k = 0; k < 3; k++)
                r.m[i][j] += a.m[i][k] * b.m[k][j];
        }
    return r;
}

static float mat3_trace(Mat3 m) {
    return m.m[0][0] + m.m[1][1] + m.m[2][2];
}

// Union for type-punning
typedef union {
    unsigned int u;
    float f;
    unsigned char bytes[4];
} Pun;

// Linked list of structs
typedef struct Entry {
    int key;
    int value;
    struct Entry *next;
} Entry;

static Entry *make_entry(int k, int v, Entry *next) {
    Entry *e = (Entry *)malloc(sizeof(Entry));
    e->key = k;
    e->value = v;
    e->next = next;
    return e;
}

static int lookup(Entry *head, int key) {
    while (head) {
        if (head->key == key) return head->value;
        head = head->next;
    }
    return -1;
}

static void free_list(Entry *head) {
    while (head) {
        Entry *tmp = head;
        head = head->next;
        free(tmp);
    }
}

// Ring buffer
typedef struct {
    int buf[16];
    int head, tail, count;
} Ring;

static void ring_init(Ring *r) { r->head = r->tail = r->count = 0; }

static int ring_push(Ring *r, int val) {
    if (r->count >= 16) return 0;
    r->buf[r->tail] = val;
    r->tail = (r->tail + 1) % 16;
    r->count++;
    return 1;
}

static int ring_pop(Ring *r, int *out) {
    if (r->count <= 0) return 0;
    *out = r->buf[r->head];
    r->head = (r->head + 1) % 16;
    r->count--;
    return 1;
}

static int approx_eq(float a, float b) {
    float d = a - b;
    if (d < 0) d = -d;
    return d < 0.001f;
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Struct Tests ===\n");

    // Vec2
    Vec2 a2 = {1.0f, 2.0f}, b2 = {3.0f, 4.0f};
    Vec2 c2 = vec2_add(a2, b2);
    CHECK(approx_eq(c2.x, 4.0f) && approx_eq(c2.y, 6.0f), "vec2_add");
    CHECK(approx_eq(vec2_dot(a2, b2), 11.0f), "vec2_dot");

    // Vec3 cross
    Vec3 i3 = {1,0,0}, j3 = {0,1,0};
    Vec3 k3 = vec3_cross(i3, j3);
    CHECK(approx_eq(k3.x, 0) && approx_eq(k3.y, 0) && approx_eq(k3.z, 1), "vec3_cross i x j = k");

    // Mat3
    Mat3 ident = mat3_identity();
    CHECK(approx_eq(mat3_trace(ident), 3.0f), "identity trace");
    Mat3 m2 = mat3_mul(ident, ident);
    CHECK(approx_eq(mat3_trace(m2), 3.0f), "I*I trace");

    Mat3 scale;
    memset(&scale, 0, sizeof(scale));
    scale.m[0][0] = 2; scale.m[1][1] = 3; scale.m[2][2] = 4;
    Mat3 sq = mat3_mul(scale, scale);
    CHECK(approx_eq(sq.m[0][0], 4) && approx_eq(sq.m[1][1], 9) && approx_eq(sq.m[2][2], 16),
          "scale^2 diagonal");

    // Union punning
    Pun p;
    p.f = 1.0f;
    CHECK(p.u == 0x3F800000, "float 1.0 == 0x3F800000");
    p.u = 0x40000000;
    CHECK(approx_eq(p.f, 2.0f), "0x40000000 == 2.0f");

    // Linked list
    Entry *head = NULL;
    for (int i = 0; i < 50; i++)
        head = make_entry(i, i * i, head);
    CHECK(lookup(head, 0) == 0, "lookup(0)");
    CHECK(lookup(head, 7) == 49, "lookup(7)");
    CHECK(lookup(head, 49) == 2401, "lookup(49)");
    CHECK(lookup(head, 100) == -1, "lookup(100) not found");
    free_list(head);

    // Ring buffer
    Ring ring;
    ring_init(&ring);
    for (int i = 0; i < 16; i++)
        CHECK(ring_push(&ring, i * 10), "ring push");
    CHECK(!ring_push(&ring, 999), "ring full");
    int val;
    CHECK(ring_pop(&ring, &val) && val == 0, "ring pop first == 0");
    CHECK(ring_pop(&ring, &val) && val == 10, "ring pop second == 10");
    ring_push(&ring, 160);
    ring_push(&ring, 170);
    // Drain remaining
    int sum = 0;
    while (ring_pop(&ring, &val)) sum += val;
    // 20+30+...+150 + 160+170 = sum of 20..150 step 10 + 330
    // 20+30+40+50+60+70+80+90+100+110+120+130+140+150 = 1190, + 330 = 1520
    CHECK(sum == 1520, "ring drain sum");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
