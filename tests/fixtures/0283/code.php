<?php 

function foo($a) {
    global ${new class() {
        public function __toString() {
            return 'p';
        }
    }};

    echo $p;
}

$p = 'why!';
$m = 'foo';

${(function() {
    return 'm';
})()}([]);
