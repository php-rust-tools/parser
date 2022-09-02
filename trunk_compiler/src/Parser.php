<?php

namespace Trunk;

use PhpParser\ParserFactory;
use PhpParser\Parser as NikicParser;

final class Parser
{
    private NikicParser $parser;

    private static ?self $instance = null;

    private function __construct()
    {
        $this->parser = (new ParserFactory)->create(ParserFactory::ONLY_PHP7);    
    }

    public function parse(string $file): array
    {
        return $this->parser->parse(file_get_contents($file));
    }

    public static function the(): self
    {
        return self::$instance ??= new self;
    }
}