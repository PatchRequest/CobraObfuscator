#include <cstdio>

class Shape {
public:
    virtual int area() const = 0;
    virtual const char *name() const = 0;
    virtual ~Shape() = default;
};

class Rectangle : public Shape {
    int w, h;
public:
    Rectangle(int w, int h) : w(w), h(h) {}
    int area() const override { return w * h; }
    const char *name() const override { return "Rect"; }
};

class Circle : public Shape {
    int r;
public:
    Circle(int r) : r(r) {}
    int area() const override { return 3 * r * r; } // approximate
    const char *name() const override { return "Circle"; }
};

class Triangle : public Shape {
    int b, h;
public:
    Triangle(int b, int h) : b(b), h(h) {}
    int area() const override { return b * h / 2; }
    const char *name() const override { return "Tri"; }
};

void print_shape(const Shape &s) {
    printf("%s: area=%d\n", s.name(), s.area());
}

int main() {
    Rectangle r(5, 3);
    Circle c(4);
    Triangle t(6, 4);

    Shape *shapes[] = {&r, &c, &t};
    for (int i = 0; i < 3; i++)
        print_shape(*shapes[i]);

    return 0;
}
