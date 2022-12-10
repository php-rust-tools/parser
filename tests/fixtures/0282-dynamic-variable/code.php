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

class f {
    public static function foo() {
        return foo([]);
    }
}

f::${(function() {
    return 'm';
})()}();
