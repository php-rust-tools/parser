<?php

class bar {};

$e = new class extends bar {
    public function bar(): parent {
        return new bar();
    }
};

