#include <stdio.h>

const int8_t TAG_INT = 0;
const int8_t TAG_BOOL = 1;
const int8_t TAG_STR = 2;
const int8_t TAG_NIL = 3;

typedef struct {
    int8_t tag;
    union {
        int i;
        bool b;
        char* s;
    } payload;
} Value;

