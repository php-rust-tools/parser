<?php 

function foo($a) {
    global $f;
    global ${$a[3]};
    global ${${$a[3]}}, ${${${$a[3]}}}, ${${${${$a[3]}}}};
    
    echo $p;
}

$p = 'why!';
$s = 'p';
$m = 's';
$f = 'm';
foo([3 => 'f']);
