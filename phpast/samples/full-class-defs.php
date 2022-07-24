<?php

class ClassConstant {
    const FOO = 1;
}

class VarDec {
    var $foo;
    var string $bar;
}

class PropDefs {
    public static $foo;
    public string $bar;
    public $baz = 100;
}

abstract class Methods {
    public function foo() {

    }

    abstract public function bar(): string;

    final public function boo() {
        
    }
}

class VisibleClassConstant {
    final protected const BAR = 2;
}