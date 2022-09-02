<?php

namespace Trunk;

use Error;
use Exception;
use PhpParser\Node;
use PhpParser\Node\Stmt\Echo_;
use PhpParser\Node\Expr\FuncCall;
use PhpParser\Node\Name;
use PhpParser\Node\Scalar\DNumber;
use PhpParser\Node\Scalar\LNumber;
use PhpParser\Node\Scalar\String_;
use Trunk\Intermediate\Instruction;
use Trunk\Intermediate\EchoInstruction;
use Trunk\Intermediate\IntegerInstruction;
use Trunk\Intermediate\FunctionCallInstruction;
use Trunk\Intermediate\ConstantStringInstruction;

class Compiler
{
    protected array $imports = [];

    protected string $source = '';

    public function compile(array $program): string
    {
        $this->init();

        $instructions = [];
        foreach ($program as $node) {
            $instructions[] = $this->toInstruction($node);
        }
        
        foreach ($instructions as $instruction) {
            $instruction->pass1($this);
        }

        $source = <<<go
        package main

        import (
            %s
        )

        func main() {
            value.Init()
            %s 
        }
        go;

        foreach ($instructions as $instruction) {
            $instruction->pass2($this);
        }

        $compiled = sprintf($source, $this->compileImports(), $this->source);

        return $compiled;
    }

    protected function compileImports(): string
    {
        return implode("\n    ", array_map(fn ($import) => '"' . $import . '"', $this->imports));
    }

    protected function toInstruction(Node $node): Instruction
    {
        if ($node instanceof Echo_) {
            $values = [];
            
            foreach ($node->exprs as $expr) {
                $values[] = $this->toInstruction($expr);
            }

            return new EchoInstruction($values);
        }

        if ($node instanceof String_) {
            return new ConstantStringInstruction($node->value);
        }

        if ($node instanceof LNumber) {
            return new IntegerInstruction($node->value);
        }

        if ($node instanceof FuncCall) {
            $args = [];
            
            foreach ($node->args as $arg) {
                $args[] = $this->toInstruction($arg->value);   
            }

            return new FunctionCallInstruction(match (true) {
                $node->name instanceof Name => $node->name,
                default => $this->toInstruction($node->name),
            }, $args);
        }

        throw new Exception('Unhandled node type: ' . $node::class);
    }

    public function addImport(string $name): void
    {
        $this->imports[] = $name;
    }

    public function push(string $code): void
    {
        $this->source .= $code;
    }

    protected function init()
    {
        $this->addImport('github.com/ryangjchandler/trunk/value');
        $this->addImport('github.com/ryangjchandler/trunk/runtime');
    }
}