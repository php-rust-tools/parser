<?php

use Trunk\BindingGen;
use Trunk\Range;

BindingGen::moduleBase('github.com/ryangjchandler/trunk');
BindingGen::module('math');
BindingGen::module('constructs');
BindingGen::module('debug');

const ARITY_ANY = -1;

// Math related functions
BindingGen::function('rand', 'math', 'Rand', new Range(0, 2));

// Debugging related functions
BindingGen::function('var_dump', 'debug', 'VarDump', ARITY_ANY);

// Language constructs
BindingGen::construct('echo', 'constructs', 'Echo', ARITY_ANY);