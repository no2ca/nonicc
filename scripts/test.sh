#!/bin/bash
cargo build
assert() {
    expected="$1"
    input="$2"
    ./target/debug/no2cc "$input" > tmp.s
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

fi

rm -f tmp*

echo OK
