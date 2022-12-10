<?php

namespace f;

class s {
    public static function foo() {
        echo "huh?\n";
    }
}

const self = new s;
const parent = new s;

class a {
    private static function foo() {
        echo "here!\n";
    }

    public function f() {
        var_dump(self);
        var_dump(parent);

        self::foo();
        (self)::foo();
        (parent)::foo();
    }
}

(new a)->f();
