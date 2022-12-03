<?php


$a = 4;

$b = match ($a) {
    1,2,3,4 => null,
    // seems weird, but PHP considers this valid.
    default, => null,
};
