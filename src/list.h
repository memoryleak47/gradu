struct list {
    T* elements;
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

void push_list(list* l, T v) {
    if (l->length == l->capacity) {
        l->capacity = max(2*l->capacity, 1);
        l->elements = realloc(l->elements, sizeof(T) * l->capacity);
    }
    l->elements[l->length] = v;
    l->length++;
}

void store_list(list* l, int i, T v) {
    assert(l->length > i);
    l->elements[i] = v;
}

T index_list(list* l, int i) {
    assert(l->length > i);
    return l->elements[i];
}


int length(list* l) {
    return l->length;
}
