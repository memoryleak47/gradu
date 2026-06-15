// <list.h>

struct list {
    $T* elements;
    int length;
    int capacity;
};

list* new_list() {
    list* l = malloc(sizeof(list));
    l->length = 0;
    l->capacity = 0;
    l->elements = nullptr;
    return l;
}

int max(int x, int y) {
    if (x > y) { return x; }
    return y;
}

void push_list(list* l, $T v) {
    if (l->length == l->capacity) {
        l->capacity = max(2*l->capacity, 1);
        l->elements = realloc(l->elements, sizeof($T) * l->capacity);
    }
    l->elements[l->length] = v;
    l->length++;
}

void store_list(list* l, int i, $T v) {
    check(l->length > i, "store_list out of range!");
    l->elements[i] = v;
}

$T index_list(list* l, int i) {
    check(l->length > i, "index_list out of range!");
    return l->elements[i];
}


int length(list* l) {
    return l->length;
}

// </list.h>
