<?php

namespace {
    function globalFunc() {}
}

namespace foo {
    $a = function () {};
    $b = function (&$b) {};
    $c = function &() {};
    $d = function &(&$b) { return $b; };
    $e = fn () => null;
    $f = fn (&$b) => null;
    $g = fn &() => null;
    $h = fn &(&$b) => $b;
}

namespace bar {
    $a = static function () {};
    $b = static function (&$b) {};
    $c = static function &() {};
    $d = static function &(&$b) { return $b; };
    $e = static fn () => null;
    $f = static fn (&$b) => null;
    $g = static fn &() => null;
    $h = static fn &(&$b) => $b;
}

namespace baz {
    function a(&$b) {}
    function &b($b) { return $b; }
    function &c() { return $b; }
}
