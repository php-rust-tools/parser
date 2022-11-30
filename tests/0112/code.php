<?php 

class foo {
    public function __construct(
        public readonly &$e,
    ) {}
}
