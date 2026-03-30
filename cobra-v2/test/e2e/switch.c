#include <stdio.h>

const char *day_name(int d) {
    switch (d) {
    case 0: return "Sun";
    case 1: return "Mon";
    case 2: return "Tue";
    case 3: return "Wed";
    case 4: return "Thu";
    case 5: return "Fri";
    case 6: return "Sat";
    default: return "???";
    }
}

int grade(int score) {
    switch (score / 10) {
    case 10: case 9: return 4;
    case 8: return 3;
    case 7: return 2;
    case 6: return 1;
    default: return 0;
    }
}

int main() {
    printf("d0=%s d3=%s d6=%s d9=%s\n",
           day_name(0), day_name(3), day_name(6), day_name(9));
    printf("g100=%d g85=%d g72=%d g55=%d\n",
           grade(100), grade(85), grade(72), grade(55));
    return 0;
}
