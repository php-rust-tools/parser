<?php

namespace Trunk\Intermediate;

use Trunk\Compiler;

class ConstantStringInstruction extends Instruction
{
    public function __construct(
        protected string $value
    )
    {
        
    }

    public function pass2(Compiler $compiler): void
    {
        $compiler->push("value.NewString(`{$this->value}`)");
    }

    public function __toString(): string
    {
        return 'string("' . str_replace("\n", '\n', $this->value) . '")';
    }
}