<?php 

$foo = give_me_foo();

$a = [
    'single' => $foo instanceof Foo,
    'multiple' => $foo instanceof Bar && $foo instanceof Baz
];
