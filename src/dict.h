// <dict.h>

struct entry {
    $K key;
    $V value;
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

void store_dict(dict* d, $K key, $V value) {
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

$V index_dict(dict* d, $K key) {
    for (int i = 0; i < d->length; i++) {
        entry* e = &d->elements[i];
        if (k_equ(e->key, key)) {
            return e->value;
        }
    }
    fail("key error");
}

// </dict.h>
