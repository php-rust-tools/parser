<?php

function foo(): never {
    try {
        bar();
    } catch (Foo|!Bar $e) {

    }
}
