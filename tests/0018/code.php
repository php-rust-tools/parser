<?php

function a(): null {
    echo "looping..\n";

    return null;
}

$bar = a(...);

foo:
    $bar();
    goto foo;
