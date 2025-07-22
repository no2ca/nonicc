#!/bin/bash
assert() {
    expected="$1"
    input="$2"
    cargo run "$input" > tmp.s
    gcc -o tmp tmp.s
    ./tmp
    actual="$?"
    if [ "$actual" = "$expected" ]; then 
        echo "$input => $actual"
    else 
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert 0 0
assert 6 6
assert 2 1+1
assert 42 ' 25 - 2 + 19 '

rm -f tmp*

echo OK
