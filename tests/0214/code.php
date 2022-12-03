<?php

interface s {}

interface foo extends s {
    public function bar(): parent;
}
