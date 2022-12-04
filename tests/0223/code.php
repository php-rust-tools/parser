<?php

#[
    A1,
    A2(),
    A3(0),
    A4(x: 1),
]
function a(
    #[A5]
    int|float $a,
    #[A6]
    $c,
    #[A7] string $b,
) {
}


#[A8, A9(), A10(foo: bar)]
class C {
    #[A11]
    public function __construct(
        #[A12]
        public readonly string $s,
    ) {}

    #[A13]
    public function m(
        #[A14] $param,
    ) {}

    #[A15]
    public $prop;
}

#[A16]
trait F {}

#[A17]
enum P {}

#[A18]
enum B: int {}

#[A19]
interface I {}

#[A20]
trait T {}

$x = #[A21] function() {};
$y = #[A22] fn() => 0;
$a = #[A23] static function() {};
$b = #[A24] static fn() => 0;
$z = new #[A25] class {
    #[A26]
    var $s;
};
