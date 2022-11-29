<?php

define('a', ($a == $b));
define('a', ($a === $b));
define('a', ($a != $b));
define('a', ($a !== $b));
define('a', ($a + $b));
define('a', ($a - $b));
define('a', ($a / $b));
define('a', ($a ^ $b));
define('a', ($a * $b));
define('a', ($a >> $b));
define('a', ($a << $b));
define('a', ($a | $b));
define('a', ($a & $b));

echo ($a + $b) * ($c / $d - ${"foo" . $c ? 4 : 3});
