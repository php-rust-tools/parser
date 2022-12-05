<?php


match ($s) {
    1 => 2,
    3, => 4,
    5,6 => 4,
    9, 123, => 4,
    _ => 43, // _ here is a constant
    default => 124,
};
