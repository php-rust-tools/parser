<?php

use Trunk\BindingGen;
use Trunk\Range;

BindingGen::moduleBase('github.com/ryangjchandler/trunk');
BindingGen::module('math');
BindingGen::module('constructs');

const ARITY_ANY = -1;
BindingGen::function('rand', 'math', 'Rand', new Range(0, 2));
BindingGen::construct('echo', 'constructs', 'Echo', ARITY_ANY);