#!/bin/bash
debug="$1"

cargo build
assert() {
    expected="$1"
    input="$2"
    if [ "$debug"="true" ]; then
        ./target/debug/no2cc "$input" -d > tmp.s
    else
        ./target/debug/no2cc "$input" > tmp.s
    fi
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

assert 0 "
foo() {
    a = 1;
    return a;
}

main() {
    foo();
    return 0;
}
"

assert 42 "
foo() {
    a = 13;
    b = 29;
    return a + b;
}

main() {
    return foo();
}
"

rm -f tmp*

echo OK