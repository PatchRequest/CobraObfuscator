#include <stdio.h>
#include <string.h>

int main() {
    const char *hello = "Hello";
    const char *world = "World";
    char buf[20];
    snprintf(buf, sizeof(buf), "%s %s", hello, world);
    printf("combined=%s len=%zu\n", buf, strlen(buf));
    return 0;
}
