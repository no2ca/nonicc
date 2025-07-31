#!/bin/bash
option="$1"
debug="$2"

cargo build
assert() {
    expected="$1"
    input="$2"
    if [ "$debug" = "true" ]; then
        ./target/debug/no2cc -d "$input" > tmp.s
        cat ./tmp.s
    else
        ./target/debug/no2cc "$input" > tmp.s
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

    assert 0 ' a = 0; '
    assert 1 ' a = 1; '
    assert 25 ' a = 5; 
b = (a + 1) * 2 / 3 + 1; 
c = b * b; c; '

    assert 2 ' foo = 1; bar = 2; foo * bar; '
    assert 4 ' foo = 3; bar = 4; foo = bar; '

    assert 1 ' return 1; '
    assert 6 ' return 1 + 2 + 3; '
    assert 6 ' a = 1; return a + 2 + 3; '
    assert 1 ' return 1; return 2; '
fi

if [ "$option" = "if-stmt" ] || [ "$option" = "all" ]; then
    # 単純なif文
    assert 10 '
    x = 1;
    if (x > 5)
        return 1;
    return 10; '

    assert 10 '
    x = 10;
    if (x < 5)
        return 1;
    return 10; '

    # 四則演算で条件分岐
    assert 1 '
    x = 5; y = 10;
    if (x - y < 0) 
        return 1;
    return 0;
    '

    assert 1 '
    x = 1; y = 2; z = 3;
    if (x + y * z > 6) 
        return 1;
    return 0;
    '

    # ローカル変数の更新
    assert 20 '
    x = 5; y = 10;
    if (y == 10)
        x = 20;
        return x;
    return 0;
    '

    # ネストしたif文
    assert 1 '
    x = 1; y = 2; z = 3;
    if (x)
        if (y)
            if (z)
                return 1;
    return 0;
    '

    assert 20 '
    x = 1; y = 1; z = 0;
    if (x)
        if (y)
            if (z)
                return 10;
    return 20;
    '
fi

if [ "$option" = "else-stmt" ] || [ "$option" = "all" ]; then
    assert 4 '
    a = 0; b = 1; c = 1;
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

    assert 3 '
    a = 1; b = 0; c = 1;
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

    assert 2 '
    a = 1; b = 1; c = 0;
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
    a = 1; b = 1; c = 1;
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
    a = 1; b = 1; c = 1;
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
    x = 1; y = 0;
    if (x) 
        if (y)
            return 0;
    return 255;
    '

    assert 1 '
    x = 1;
    if (x == 0)
        return 0;
    else if (x == 1)
        return 1;
    else 
        return 3;
    '

    assert 30 '
    x = 3;
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
    a = 1;
    if (a == 1)
        a = a + 1;
    else if (a == 1)
        a = a + 2;
    else if (a == 1)
        a = a + 3;
    return a;
    '

fi

rm -f tmp*

echo OK
