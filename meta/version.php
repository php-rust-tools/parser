#!/usr/bin/env php
<?php

$files = [
    __DIR__ . '/../trunk_compiler/Cargo.toml',
    __DIR__ . '/../trunk_parser/Cargo.toml',
    __DIR__ . '/../trunk_lexer/Cargo.toml',
];

$version = readline('What would you like the new version to be?     ');

foreach ($files as $file) {
    echo "Updating {$file}.\n";
    
    file_put_contents(
        $file, 
        preg_replace('/version = "(.*)"/', 'version = "' . $version . '"', file_get_contents($file))
    );
}

echo "Version updated to {$version}.\n";