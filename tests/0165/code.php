<?php

class Foo {
    public function __construct(
        public callable $s,
    ) {}
}