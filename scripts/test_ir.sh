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

if [ "$1" = "all" ]; then
assert 0 "0;"
assert 42 "42;"

assert 2 " 1; 2; "
assert 0 " 5; 4; 3; 2; 1; 0; "

assert 42 " 3 + 39; "
assert 42 " 998244353 - 998244311; "

assert 42 " 6 * 7; "
assert 42 " 300 / 7; "

assert 8 " (1 + 2) * 3 - (4 + 5) - 6 / 7 + 8; "
assert 42 " ((((42) - 42)) + 42); "

assert 1 " 0 - -1; "
assert 1 " 0 + +1; "
assert 42 " (1 + +2 / -3) * (-4 / -5 - -6 * +7); "

assert 0 ' 1 != 1; '
assert 1 ' 1 == 1; '
assert 0 ' 1 < 1; '
assert 1 ' 1 <= 1; '
assert 0 ' 1 > 1; '
assert 1 ' 1 >= 1; '
fi

rm -f tmp*

echo OK