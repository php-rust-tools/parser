<?php

class MyParentClass {}

class MyClass extends MyParentClass {
    const bool A = true;
    const string B = '';
    const ?string C = null;
    const null|string D = '';
    const int E = 1;
    const float F = 2;
    const \Stringable|null G = null;
    const (\Stringable&\Countable)|null H = null;
    const self|null I = null;
    const parent|null J = null;
    const iterable K = [];
    const null L = null;
    const false M = false;
    const true N = true;
    const static|null O = null;
    const mixed P = null;
    const Q = '';
    public const string R = '';
    protected const string S = '';
    private const string T = '';
    const string U = '', V = '';
}

enum MyEnum {
    const MyEnum A = MyEnum::Foo;
    const self B = self::Foo;
    const static C = MyEnum::Foo;

    case FOO;
}
