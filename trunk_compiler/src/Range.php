<?php

namespace Trunk;

final class Range
{
    private array $range = [];

    public function __construct(public readonly int $start, public readonly int $end)
    {
        $this->range = range($start, $end);
    }

    public function start(): int
    {
        return $this->start;    
    }

    public function end(): int
    {
        return $this->end;
    }

    public function includes(int $subject): bool
    {
        return in_array($subject, $this->range);
    }
}