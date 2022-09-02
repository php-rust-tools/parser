<?php

namespace Trunk\Intermediate;

use Trunk\Compiler;

class IntegerInstruction extends Instruction
{
    public function __construct(
        protected int $value,
    ) {}

    public function pass2(Compiler $compiler): void
    {
        $compiler->push("value.NewInt({$this->value})");
    }

    public function __toString(): string
    {
        return "int({$this->value})";
    }
}