<?php

namespace Trunk\Intermediate;

use Trunk\BindingGen;
use Trunk\Compiler;

class EchoInstruction extends Instruction
{
    public function __construct(
        protected array $values,
    ) {}

    public function pass1(Compiler $compiler): void
    {
        assert([$module, $function, $_] = BindingGen::getConstructBinding('echo'));

        $compiler->addImport(BindingGen::resolveModuleName($module));

        foreach ($this->values as $value) {
            $value->pass1($compiler);
        }
    }

    public function pass2(Compiler $compiler): void
    {
        assert([$module, $function, $_] = BindingGen::getConstructBinding('echo'));

        $compiler->push("{$module}.{$function}(");
        foreach ($this->values as $value) {
            $value->pass2($compiler);
        }
        $compiler->push(')');
    }

    public function __toString(): string
    {
        return 'echo(' . count($this->values) . ')    ' . implode(' | ', array_map(fn (Instruction $value) => (string) $value, $this->values));
    }
}