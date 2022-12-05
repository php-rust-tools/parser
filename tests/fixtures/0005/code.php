<?php

function foo(): never {
    try {
        bar();
    } catch {

    }
}
