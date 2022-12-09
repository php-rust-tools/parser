<?php

#[A, B]
#[C, D]
interface A extends B, C {
    #[R]
    const F = 344;

    #[R]
    public const O = 344;

    #[R]
    #[P]
    final public const R = 344, P = 214;

    #[R]
    #[P]
    final const M = 34;

    #[M]
    public function bar(): void;

    #[Q]
    #[S]
    public static function baz(): void;
}
