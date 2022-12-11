<?php

enum EnumWithUnserialize {
    public function __unserialize(array $data) {}
}
