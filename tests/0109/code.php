<?php 

class foo {
    public function __construct(
        public string $a,
        public readonly int $b,
        public readonly float &$c,
        &...$e,
    ) {}
}
