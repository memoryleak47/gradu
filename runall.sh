#!/bin/bash

function run() {
    rm -f exe exe.c
    cargo r -q -- "$1" 2>/dev/null
}

function output() {
    cat "$1" | grep "# OUT: " | sed 's/.*# OUT: //'
}

for x in $(find examples -type f -name "*.gradu" )
do
    a=$(run "$x")
    b=$(output "$x")
    if [ "$a" == "$b" ]; then
        echo "$x passed!"
    else
        echo "$x failed!"
        echo
        echo "compiler output:"
        echo "$a"
        echo
        echo "vs"
        echo
        echo "expected output:"
        echo "$b"
    fi
done
    
