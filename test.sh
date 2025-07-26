#!/bin/bash
assert() {
    expected="$1"
    input="$2"
    cargo run -r "$input" > tmp.s
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

option="$1"

if [ "$option" = "all" ]; then
    assert 0 '0;'

    assert 2 '1+1;'

    assert 42 ' 25 - 2 + 19; '
    assert 21 ' 42 / 2; '

    assert 2 ' 1 + (2 + 3) / 4; '
    assert 6 '(1) / 2 + (2 * 3); '

    assert 0 ' +1 + -1; '
    assert 0 ' (-1) - -1; '
    assert 1 ' (-1) * -1; '

    assert 0 ' 1 != 1; '
    assert 1 ' 1 == 1; '
    assert 0 ' 1 < 1; '
    assert 1 ' 1 <= 1; '
    assert 0 ' 1 > 1; '
    assert 1 ' 1 >= 1; '
fi

assert 0 ' a = 0; '
assert 1 ' a = 1; '
assert 25 ' a = 5; 
b = (a + 1) * 2 / 3 + 1; 
c = b * b; c; '

rm -f tmp*

echo OK
