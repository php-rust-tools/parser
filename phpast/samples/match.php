<?php

match ($expr) {
    'foo' => 'bar',
    'baz', 'car', 'bob' => 'foop',
    default => null,
};