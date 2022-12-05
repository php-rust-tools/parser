<?php

class Foo
{
    function bar()
    {
        static function (self $foo) {};
    }
}
