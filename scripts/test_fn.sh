#!/bin/bash
debug="$1"

cargo build
assert() {
    expected="$1"
    input="$2"
    if [ "$debug" = "true" ]; then
        ./target/debug/nonicc "$input" -d > tmp.s
        cat tmp.s
    else
        ./target/debug/nonicc "$input" > tmp.s
    fi
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
int foo() {
    int a;
    a = 1;
    return a;
}

int main() {
    foo();
    return 0;
}
"

assert 42 "
int foo() {
    int a; int b;
    a = 13;
    b = 29;
    return a + b;
}

int main() {
    return foo();
}
"

assert 3 "
int foo(int a, int b) {
    return a + b;
}

int main() {
    return foo(1, 2);
}
"

assert 36 "
int bar(int a) {
    return a + a;
}

int foo(int a) {
    return bar(a) * bar(a);
}

int main() {
    return foo(3);
}
"

assert 15 "
int foo(int a, int b, int c, int d, int e) {
    return a + b + c + d + e;
}

int main() {
    return foo(1, 2, 3, 4, 5);
}
"

assert 6 "
int fact(int a) {
    if (a <= 1) return 1;
    return a * fact(a - 1);
}

int main() {
    return fact(3);
}
"

assert 8 "
int fib(int n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    return fib(6);
}
"

assert 55 "
int sum_to(int n) {
    if (n <= 0) return 0;
    return n + sum_to(n - 1);
}

int main() {
    return sum_to(10);
}
"

assert 0 "
int fact(int n, int acc) {
    if (n <= 1) return acc;
    return fact(n - 1, acc * n);
}

int main() {
   return fact(16, 2);
}
"

assert 125 "
int ack(int m, int n) {
    if (m == 0) return n + 1;
    if (n == 0) return ack(m - 1, 1);
    return ack(m - 1, ack(m, n - 1));
}

int main() {
    return ack(3, 4);
}
"

assert 42 "
int sum(int a, int b, int c, int d, int e, int f) {
    return a + b + c + d + e + f;
}

int main() {
    return sum(6, 7, 6, 8, 6, 9);
}
"

rm -f tmp*

echo OK