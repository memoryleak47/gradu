// <dict.h>

// TODO: this should be a parameter
bool k_equ(__K__ k1, __K__ k2) {
    return is_equal(k1, k2);
}

struct entry {
    __K__ key;
    __V__ value;
};

struct dict {
    entry* elements;
    int length;
    int capacity;
};

dict* new_dict() {
    dict* d = malloc(sizeof(dict));
    d->length = 0;
    d->capacity = 0;
    d->elements = nullptr;
    return d;
}

void store_dict(dict* d, __K__ key, __V__ value) {
    for (int i = 0; i < d->length; i++) {
        entry* e = &d->elements[i];
        if (k_equ(e->key, key)) {
            e->value = value;
            return;
        }
    }

    if (d->length == d->capacity) {
        d->capacity = max(2*d->capacity, 1);
        d->elements = realloc(d->elements, sizeof(entry) * d->capacity);
    }

    entry e;
    e.key = key;
    e.value = value;

    d->elements[d->length] = e;
    d->length++;
}

__V__ index_dict(dict* d, __K__ key) {
    for (int i = 0; i < d->length; i++) {
        entry* e = &d->elements[i];
        if (k_equ(e->key, key)) {
            return e->value;
        }
    }
    fail("key error");
}

// </dict.h>
