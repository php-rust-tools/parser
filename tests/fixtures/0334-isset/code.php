<?php

isset($a);
isset($a, $b, $c);
isset($this->name);
isset($this->{"name"});
isset($this->data[$profile->getType()]);
isset(Foo::$data[$bar->baz()]);
isset(static::$data[$bar->baz()]);
isset(self::$data[$bar->baz()]);
isset(\func_get_args()[0]);
