<?php

function bar(
    A|(B&C) $i
): (B&C)|A {
    return $i;
}
