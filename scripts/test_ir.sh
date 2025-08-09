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

assert 0 "return 0;"
assert 42 "return 42;"

assert 2 " 1; return 2; "
assert 0 " 5; 4; 3; 2; 1; return 0; "

assert 42 " return 3 + 39; "
assert 42 " return 998244353 - 998244311; "

assert 42 " return 6 * 7; "
assert 42 " return 300 / 7; "

assert 8 " return (1 + 2) * 3 - (4 + 5) - 6 / 7 + 8; "
assert 42 " return ((((42) - 42)) + 42); "

assert 1 " return 0 - -1; "
assert 1 " return 0 + +1; "
assert 42 " return (1 + +2 / -3) * (-4 / -5 - -6 * +7); "

assert 0 ' return 1 != 1; '
assert 1 ' return 1 == 1; '
assert 0 ' return 1 < 1; '
assert 1 ' return 1 <= 1; '
assert 0 ' return 1 > 1; '
assert 1 ' return 1 >= 1; '

assert 42 " a = 42; return a; "
assert 2 " a = 1; a = a + 1; return a; "
assert 3 " a = 1; b = 2; a = a + b; return a; "
assert 3 " a = 1; b = 2; c = a + b; return c; "
assert 0 ' a = 0; return a; '
assert 1 ' a = 1; return a; '
assert 25 ' a = 5; 
b = (a + 1) * 2 / 3 + 1; 
c = b * b; return c; '

fi

assert 2 ' foo = 1; bar = 2; return foo * bar; '
assert 4 ' foo = 3; bar = 4; foo = bar; return foo;'
assert 21 " a = 1; b = 2; c = 3; d = 4; e = 5; f = 6; return a+b+c+d+e+f; "

rm -f tmp*

echo OK