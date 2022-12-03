<?php

$e = new class {
    public function bar(): parent {
        exit(1);
    }
};

