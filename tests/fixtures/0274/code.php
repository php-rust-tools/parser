<?php

class a {
    public function foo() {
        $q = function() {
            return parent::bar();
        };
    }
}
