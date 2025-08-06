#!/bin/bash

cargo build
assert() {
    expected="$1"
    input="$2"
    ./target/debug/no2cc "$input" -d -i > tmp.s
    cat tmp.s
    gcc -z noexecstack -o tmp tmp.s
    ./tmp
    actual="$?"
    if [ "$actual" = "$expected" ]; then 
        echo "$input => $actual"
    else 
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert 0 "0;"
assert 42 "42;"

assert 2 " 1; 2; "

rm -f tmp*

echo OK