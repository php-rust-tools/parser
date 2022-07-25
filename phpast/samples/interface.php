<?php

interface Foo {}
interface Bar extends Foo {}
interface Car {}
interface Baz extends Bar, Car {}

interface Bob {
    public function bar(): string;
    function boo();
}