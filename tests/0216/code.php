<?php

class foo {
    public function bar(): parent {
        exit(1);
    }
}
