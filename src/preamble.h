#include <stdio.h>

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

