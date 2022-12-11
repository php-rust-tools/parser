<?php

enum EnumWithCall {
    public function __call($k, $v) {}
}
