<?php

interface A {}
interface B {}
interface C {}
interface D {}

function foo(A|(B&C&D) $a): A&(B|C|D) {
    exit(0);
}
