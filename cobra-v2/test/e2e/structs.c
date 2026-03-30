#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int x, y;
} Point;

Point add_points(Point a, Point b) {
    Point r = {a.x + b.x, a.y + b.y};
    return r;
}

int dot_product(Point a, Point b) {
    return a.x * b.x + a.y * b.y;
}

typedef struct Node {
    int val;
    struct Node *next;
} Node;

Node *make_list(int n) {
    Node *head = NULL;
    for (int i = n; i >= 1; i--) {
        Node *node = (Node *)malloc(sizeof(Node));
        node->val = i;
        node->next = head;
        head = node;
    }
    return head;
}

int sum_list(Node *head) {
    int s = 0;
    for (Node *n = head; n; n = n->next)
        s += n->val;
    return s;
}

void free_list(Node *head) {
    while (head) {
        Node *tmp = head;
        head = head->next;
        free(tmp);
    }
}

int main() {
    Point a = {3, 4}, b = {1, 2};
    Point c = add_points(a, b);
    printf("add=(%d,%d) dot=%d\n", c.x, c.y, dot_product(a, b));

    Node *list = make_list(5);
    printf("listsum=%d\n", sum_list(list));
    free_list(list);
    return 0;
}
