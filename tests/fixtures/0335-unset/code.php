<?php

unset($a);
unset($a, $b, $c);
unset($this->name);
unset($this->{"name"});
unset($this->data[$profile->getType()]);
unset(Foo::$data[$bar->baz()]);
unset(static::$data[$bar->baz()]);
unset(self::$data[$bar->baz()]);
unset(\func_get_args()[0]);
