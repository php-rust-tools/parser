<?php

interface enum extends enum, from {}

class enum extends enum implements enum, from {}
class from extends from implements Foo\enum, \Bar\Baz\from {}
