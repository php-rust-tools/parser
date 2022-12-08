<?php

#[Super::<Foo>(bar: new Super(), baz: new Super::<string>())]
function super<T super M>(
    Bar<T> $s
): T {

}

super::<string>();

