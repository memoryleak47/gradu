#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#define TAG_INT 0
#define TAG_BOOL 1
#define TAG_STR 2
#define TAG_NIL 3
#define TAG_LIST 4

#define int_to_value(x) ((Value) { .tag = TAG_INT, .payload.i = x })
#define bool_to_value(x) ((Value) { .tag = TAG_BOOL, .payload.b = x })
#define str_to_value(x) ((Value) { .tag = TAG_STR, .payload.s = x })
#define nil_to_value() ((Value) { .tag = TAG_NIL })
#define list_to_value(x) ((Value) { .tag = TAG_LIST, .payload.l = x })

typedef struct Value Value;
typedef struct list list;

struct Value {
    char tag;
    union {
        int i;
        bool b;
        char* s;
        list* l;
        void* f;
    } payload;
};

void check(bool b, char* s) {
    if (!b) {
        printf("ERROR: %s\n", s);
        exit(1);
    }
}

void print_value(Value v) {
    if (v.tag == TAG_INT) {
        printf("%i\n", v.payload.i);
    } else if (v.tag == TAG_BOOL) {
        if (v.payload.b) {
            printf("true\n");
        } else {
            printf("false\n");
        }
    } else if (v.tag == TAG_STR) {
        printf("%s\n", v.payload.s);
    } else if (v.tag == TAG_NIL) {
        printf("nil\n");
    } else if (v.tag >= 10) {
        printf("<function>\n");
    } else {
        check(false, "unknown value!");
    }
}

Value input() {
    char buf[1024];
    if (!fgets(buf, sizeof(buf), stdin)) {
        fprintf(stderr, "Fatal error: Failed to read from stdin\n");
        exit(EXIT_FAILURE);
    }
    buf[strcspn(buf, "\n")] = '\0';

    if (strcmp(buf, "true") == 0)  return (Value){.tag = TAG_BOOL, .payload.b = true};
    if (strcmp(buf, "false") == 0) return (Value){.tag = TAG_BOOL, .payload.b = false};

    size_t len = strlen(buf);
    if (len >= 2 && buf[0] == '"' && buf[len - 1] == '"') {
        return (Value){.tag = TAG_STR, .payload.s = strndup(buf + 1, len - 2)};
    }

    char* endptr;
    long val = strtol(buf, &endptr, 10);
    if (endptr != buf && *endptr == '\0') {
        return (Value) {.tag = TAG_INT, .payload.i = (int)val};
    }

    fprintf(stderr, "Fatal error: Invalid input '%s'\n", buf);
    exit(EXIT_FAILURE);
}

int value_to_int(Value v) {
    check(v.tag == TAG_INT, "value_to_int failed!");
    return v.payload.i;
}

char* value_to_str(Value v) {
    check(v.tag == TAG_STR, "value_to_str failed!");
    return v.payload.s;
}

bool value_to_bool(Value v) {
    check(v.tag == TAG_BOOL, "value_to_bool failed!");
    return v.payload.b;
}

list* value_to_list(Value v) {
    check(v.tag == TAG_LIST, "value_to_list failed!");
    return v.payload.l;
}

bool is_equal(Value v1, Value v2) {
    if (v1.tag != v2.tag) { return false; }
    switch (v1.tag) {
        case TAG_INT: return v1.payload.i == v2.payload.i;
        case TAG_BOOL: return v1.payload.b == v2.payload.b;
        case TAG_STR: return strcmp(v1.payload.s, v2.payload.s) == 0;
        case TAG_NIL: return true;
        case TAG_LIST: return v1.payload.l == v2.payload.l; // ptr compare
        default: return v1.payload.f == v2.payload.f; // the remaining tags are for functions.
    }
}
