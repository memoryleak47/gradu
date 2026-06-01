#include <stdio.h>
#include <assert.h>

const char TAG_INT = 0;
const char TAG_BOOL = 1;
const char TAG_STR = 2;
const char TAG_NIL = 3;

typedef struct {
    char tag;
    union {
        int i;
        bool b;
        char* s;
    } payload;
} Value;

void print_value(Value v) {
    if (v.tag == TAG_INT) {
        printf("%i\n", v.payload.i);
    } else if (v.tag == TAG_BOOL) {
        printf("%b\n", v.payload.b);
    } else if (v.tag == TAG_STR) {
        printf("%s\n", v.payload.s);
    } else if (v.tag == TAG_NIL) {
        printf("nil\n");
    } else {
        printf("unknown value!\n");
        assert(false);
    }
}

