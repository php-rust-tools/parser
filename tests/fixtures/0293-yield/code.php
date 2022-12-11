<?php

function foo(): iterable {
    yield 1;
    yield;
    yield 123 => 41;
    yield $a;

    yield ++$a;
    yield $b++;

    yield ++$a => $a ? $m : $s;
    yield ++$a ? $m : $s => ++$a ? $m : $s;

    yield $a++ => $a ? $m : $s;
    yield $a++ ? $m : $s => $a++ ? $m : $s;
}
