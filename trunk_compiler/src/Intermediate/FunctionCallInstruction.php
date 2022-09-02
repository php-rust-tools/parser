<?php

namespace Trunk\Intermediate;

use PhpParser\Node\Name;
use Trunk\BindingGen;
use Trunk\Compiler;

class FunctionCallInstruction extends Instruction
{
    public function __construct(
        protected Instruction | Name $function,
        protected array $args = [],
    ) {}

    public function pass1(Compiler $compiler): void
    {
        if ($this->function instanceof Name && $binding = BindingGen::getFunctionBinding($this->function->toString())) {
            $compiler->addImport(BindingGen::resolveModuleName($binding[0]));
        }
    }

    public function pass2(Compiler $compiler): void
    {
        $compiler->push($this->getCompiledFunctionName() . '(runtime.NewArgs(');
        foreach ($this->args as $i => $arg) {
            if ($i > 0) {
                $compiler->push(', ');
            }

            $arg->pass2($compiler);
        }
        $compiler->push('))');
    }

    protected function getCompiledFunctionName(): string
    {
        if ($this->function instanceof Name && $binding = BindingGen::getFunctionBinding($this->function->toString())) {
            return "{$binding[0]}.{$binding[1]}";
        }

        dd('todo', __FILE__ . '@' . __LINE__);
    }

    public function __toString(): string
    {
        return 'call';
    }
}