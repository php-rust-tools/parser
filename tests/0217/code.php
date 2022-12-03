<?php

class s {}

class foo extends s {
    public function bar(): parent {
        return new s();
    }
}
