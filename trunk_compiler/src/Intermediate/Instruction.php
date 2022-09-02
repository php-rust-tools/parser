<?php

namespace Trunk\Intermediate;

use Stringable;
use Trunk\Compiler;

abstract class Instruction implements Stringable
{
    public function pass1(Compiler $compiler): void
    {
        //
    }

    public function pass2(Compiler $compiler): void
    {
        //
    }
}