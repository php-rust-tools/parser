<?php

class A<T> {}
class A<T as string|int> {}
class A<+T> {}
class A<+T as string|int> {}

interface A<T> {}
interface A<T as string|int> {}
interface A<+T> {}
interface A<+T as string|int> {}

function foo<T>(T $a) {}
function foo<T as string|int>(T $a) {}
function foo<+T>(T $a) {}
function foo<+T as string|int>(T $a) {}

class B<Ta, Tb as string|int> {}
class B<Ta as string|int, Tb> {}
class B<+Ta, Tb> {}
class B<Ts, +T as string|int> {}

interface B<Ta, Tb as string|int> {}
interface B<Ta as string|int, Tb> {}
interface B<+Ta, Tb> {}
interface B<Ts, +T as string|int> {}

function foo<Ta, Tb as string|int>(T $a) {}
function foo<Ta, Tb = string|int>(T $a) {}
function foo<Ta, Tb super string|int>(T $a) {}
function foo<Ta, +Tb as string|int>(T $a) {}
function foo<Ta, +Tb = string|int>(T $a) {}
function foo<Ta, +Tb super string|int>(T $a) {}
function foo<Ta, -Tb as string|int>(T $a) {}
function foo<Ta, -Tb = string|int>(T $a) {}
function foo<Ta, -Tb super string|int>(T $a) {}
function foo<Ta as string|int, Tb>(T $a) {}
function foo<Ta = string|int, Tb>(T $a) {}
function foo<Ta super string|int, Tb>(T $a) {}
function foo<+Ta, Tb>(T $a) {}
function foo<Ts, +T as string|int>(T $a) {}

class B<Ts, +T as string|int> {
    public function bar<M>(): string {
        return self::baz::<M, int>();
    }

    public static function baz<M, S>(): string {}
}

function bar<
    T, +T, -T,
    T as Foo, T super Foo, T = Foo,
    +T as Foo, +T super Foo, +T = Foo,
    -T as Foo, -T super Foo, -T = Foo
>(
    Bar<A|B|(C&D)> $bar,
    Bar<A&B&(C|D)> $baz,
): Foo<Bar<Baz<string>|M>&R<string, int, O<S&W>>> {
    exit(0);
}

$a = B::baz::<M>();

$a = new A::<string>();
$a = new A::<string, int, float>();
$a = foo::<string>();
$a = foo::<string, int, float>();
$a = new Collection::<Foo&Bar>();
$a = new Collection::<Foo|Bar>();
$a = new Collection::<string|int, Foo&Bar>();
$a = new Collection::<string|int|float|bool, Foo|Bar>();
