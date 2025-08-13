#!/bin/bash
debug="$1"

cargo build
assert() {
    expected="$1"
    input="$2"
    if [ "$debug"="true" ]; then
        ./target/debug/nonicc "$input" -d > tmp.s
    else
        ./target/debug/nonicc "$input" > tmp.s
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

assert 3 "
foo(a, b) {
    return a + b;
}

main() {
    return foo(1, 2);
}
"

assert 36 "
bar(a) {
    return a + a;
}

foo(a) {
    return bar(a) * bar(a);
}

main() {
    return foo(3);
}
"

assert 15 "
foo(a, b, c, d, e) {
    return a + b + c + d + e;
}

main() {
    return foo(1, 2, 3, 4, 5);
}
"

assert 6 "
fact(a) {
    if (a <= 1) return 1;
    return a * fact(a - 1);
}

main() {
    return fact(3);
}
"

assert 8 "
fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

main() {
    return fib(6);
}
"

assert 55 "
sum_to(n) {
    if (n <= 0) return 0;
    return n + sum_to(n - 1);
}

main() {
    return sum_to(10);
}
"

assert 0 "
fact(n, acc) {
    if (n <= 1) return acc;
    return fact(n - 1, acc * n);
}

main() {
   return fact(16, 2);
}
"

assert 125 "
ack(m, n) {
    if (m == 0) return n + 1;
    if (n == 0) return ack(m - 1, 1);
    return ack(m - 1, ack(m, n - 1));
}

main() {
    return ack(3, 4);
}
"

rm -f tmp*

echo OK