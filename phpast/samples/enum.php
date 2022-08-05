<?php

enum Foo {
    case Bar;
}

enum Bar: string {
    case Baz = 'car';
}

enum Baz: int {
    case Caz = 'boo';
}

enum Foo implements Bob {
    
}