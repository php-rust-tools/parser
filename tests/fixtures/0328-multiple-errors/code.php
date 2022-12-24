<?php

function bar(): a&(b|c|(d&eeeef)) {}

function bar(): b|c|(d&eeeef) {}

function bar(): d&eeeef {}

function foo(): ?never {}

enum Foo: int {
  case Bar = 1;
  case Baz;
}

enum Bar {
  case Baz;
  case Qux = 1;
}

enum Baz: int {
    public function __construct() {}
}

enum Qux: int {
    public function __set($_, $_) {}
}

class Hello {
    private protected public readonly static $foo;
}

self:
    goto interface;
    parent:
        goto class;
            static:

const foreach = 1;

class Foreach {}
class For {}
