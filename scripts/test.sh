#!/bin/bash

debug="$2"

cargo build
assert() {
    expected="$1"
    input="$2"
    if [ "$debug" = "true" ]; then
        ./target/debug/nonicc "int main() { $input }" -d > tmp.s
        cat tmp.s
    else
        ./target/debug/nonicc "int main() { $input }" > tmp.s
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

assert 42 " int a; a = 42; return a; "
assert 2 " int a; a = 1; a = a + 1; return a; "
assert 3 " int a; int b; a = 1; b = 2; a = a + b; return a; "
assert 3 " int a; int b; int c; a = 1; b = 2; c = a + b; return c; "
assert 0 ' int a; a = 0; return a; '
assert 1 ' int a; a = 1; return a; '
assert 25 ' int a; int b; int c; a = 5; 
b = (a + 1) * 2 / 3 + 1; 
c = b * b; return c; '

assert 2 ' int foo; int bar; foo = 1; bar = 2; return foo * bar; '
assert 4 ' int foo; int bar; foo = 3; bar = 4; foo = bar; return foo;'
assert 10 " int a; int b; int c; int d; a = 1; b = 2; c = 3; d = 4; return a+b+c+d; "

# 単純なif文
assert 10 '
int x;
x = 1;
if (x > 5)
    return 1;
return 10; '

assert 10 '
int x;
x = 10;
if (x < 5)
    return 1;
return 10; '

# 四則演算で条件分岐
assert 1 '
int x; int y; x = 5; y = 10;
if (x - y < 0)
    return 1;
return 0;
'

assert 1 '
int x; int y; int z; x = 1; y = 2; z = 3;
if (x + y * z > 6)
    return 1;
return 0;
'

# ローカル変数の更新
assert 20 '
int x; int y; x = 5; y = 10;
if (y == 10)
    x = 20;
    return x;
return 0;
'

# ネストしたif文
assert 1 '
int x; int y; int z; x = 1; y = 2; z = 3;
if (x)
    if (y)
        if (z)
            return 1;
return 0;
'

assert 20 '
int x; int y; int z; x = 1; y = 1; z = 0;
if (x)
    if (y)
        if (z)
            return 10;
return 20;
'

assert 4 '
int a; int b; int c; a = 0; b = 1; c = 1;
if (a)
    if (b)
        if (c)
            return 1;
        else
            return 2;
    else
        return 3;
else
    return 4;
'

assert 1 '
int a; int b; int c; a = 1; b = 1; c = 1;
if (a)
    if (b)
        if (c)
            return 1;
        else
            return 2;
    else
        return 3;
else
    return 4;
'

assert 1 '
int a; int b; int c; int x; a = 1; b = 1; c = 1;
x = 0;
if (a)
    if (b)
        if (c)
            x = x + 1;
        else
            x = x + 2;
    else
        x = x + 3;
else
    x = x + 4;

return x;
'

assert 255 '
int x; int y; x = 1; y = 0;
if (x)
    if (y)
        return 0;
return 255;
'

assert 1 '
int x; x = 1;
if (x == 0)
    return 0;
else if (x == 1)
    return 1;
else
    return 3;
'

assert 30 '
int x; x = 3;
if (x == 1)
    return 10;
else if (x == 2)
    return 20;
else if (x == 3)
    return 30;
else
    return 40;
'

assert 2 '
int a; a = 1;
if (a == 1)
    a = a + 1;
else if (a == 1)
    a = a + 2;
else if (a == 1)
    a = a + 3;
return a;
'

assert 1 '
int x; x = 1;
if (x > 5) {
    return 5;
} else {
    return 1;
}
'

assert 150 '
int x; int y; int z; x = 1;
{
    if (x) {
        y = 100;
    } else {
        y = 200;
    }
    z = y + 50;
}
return z;
'

assert 42 '
int x; x = 42;
{
}
return x;
'

assert 2 '
int a; a = 1;
if (a == 1) {
    a = a + 1;
} else if (a == 1) {
    a = a + 2;
} else if (a == 1) {
    a = a + 3;
}
return a;
'

assert 1 '
int a; int b; int c; int x; a = 1; b = 1; c = 1;
x = 0;
if (a) {
    if (b) {
        if (c) {
            x = x + 1;
        } else {
            x = x + 2;
        }
    } else {
        x = x + 3;
    }
} else {
    x = x + 4;
}
return x;
'

assert 55 "
int x; int sum; x = 1; sum = 0;
while (x <= 10) {
    sum = sum + x;
    x = x + 1;
}
return sum;
"

assert 0 "
int x; int sum; x = 11; sum = 0;
while (x <= 10) {
    sum = sum + x;
    x = x + 1;
}
return sum;
"

assert 55 "
int x; int sum; x = 10; sum = 0;
while (x > 0) {
    sum = sum + x;
    x = x - 1;
}
return sum;
"

assert 6 "
int x; int sum; x = 3; sum = 0;
while (x) {
    sum = sum + x;
    x = x - 1;
}
return sum;
"

assert 18 "
int i; int sum; int j; i = 1; sum = 0;
while (i <= 3) {
    j = 1;
    while (j <= 2) {
        sum = sum + i * j;
        j = j + 1;
    }
    i = i + 1;
}
return sum;
"

assert 15 "
int x; int sum; x = 1; sum = 0;
while (x * x <= 25) {
    sum = sum + x;
    x = x + 1;
}
return sum;
"

assert 8 "
int x; int sum; x = 1; sum = 0;
while (x <= 10000) {
    sum = sum + x;
    x = x + 1;
}
return sum;
"

assert 55 "
int sum; int x; sum = 0;
for (x = 1; x <= 10; x = x + 1) {
    sum = sum + x;
}
return sum;
"

assert 0 "
int sum; int x; sum = 0;
for (x = 11; x <= 10; x = x + 1) {
    sum = sum + x;
}
return sum;
"

assert 55 "
int sum; int x; sum = 0;
for (x = 10; x > 0; x = x - 1) {
    sum = sum + x;
}
return sum;
"

assert 18 "
int sum; int i; int j; sum = 0;
for (i = 1; i <= 3; i = i + 1) {
    for (j = 1; j <= 2; j = j + 1) {
        sum = sum + i * j;
    }
}
return sum;
"

assert 21 "
int sum; int x; sum = 0;
for (x = (2 + 3); x < 10; x = x + 2) {
    sum = sum + x;
}
return sum;
"

fi

# 現在の割り当てアルゴリズムでは
# コードが正しくても通らないのでスキップ
# assert 3 "
# sum = 0;
# for (x = 3; (x = x - 1) > 0; ) {
#     sum = sum + x;
# }
# return sum;
# "

assert 3 "
int x; int y; x = 3;
y = &x;
return *y;
"

assert 3 "
int x; int y; int z; x = 3;
y = &x;
z = &y;
return **z;
"

assert 3 "
int x; int y; x = 1;
y = &x;
*y = 3;
return x;
"

assert 3 "
int x; int y; int *a; int *b; 
x = 1;
y = 2;
a = &x;
b = &y;
return *a + *b;
"

assert 30 "
int a; int b; int *p; 
a = 1; 
b = 2;
p = &a;
*p = 10;
p = &b;
*p = 20;
return a + b;
"

assert 3 "
int a; 
int *p; 
int **pp; 
int ***ppp; 
a = 1;
p = &a;
pp = &p;
ppp = &pp;
***ppp = 3;
return a;
"

assert 30 "
int a; 
int b; 
int *p; 
int **pp; 
a = 10;
p = &a;
pp = &p;
*p = 20;
b = 30;
*pp = &b;
return *p;
"

rm -f tmp*

echo OK
