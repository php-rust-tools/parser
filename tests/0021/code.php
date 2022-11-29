<?php

if ($a):
    $a;
endif;

if ($a):
    $a;
else:
    $b;
endif;

if (true):
    $a;
elseif ($foo->bar() && $baz->bar?->qux()):
    $b;
endif;

if (true):
    $a;
elseif (true):
    $b;
elseif (true):
    $c;
endif;
