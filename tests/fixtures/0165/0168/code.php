<?php

class Foo {
    public function __construct(
        public string|int|callable $s,
    ) {}
}
