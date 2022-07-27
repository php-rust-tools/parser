<?php

switch ($foo) {
    case 'foo':
        break;
    case 'bar':
    case 'baz':
        echo $foo;
    default:
        break;
}