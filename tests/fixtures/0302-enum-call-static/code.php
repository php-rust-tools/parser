<?php

enum EnumWithCallStatic {
    public static function __callStatic($k, $v) {}
}
