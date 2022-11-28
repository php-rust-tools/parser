<?php

function foo(string $a = "", array $b = []): never {
    exit(1);
}

function bar(int $a, float $b, string $c, true $d, false $e, null $f): null|string|int|float {
    return null;
}
