<?php

class foo {
    use a, b, c {
        a::s insteadof b, c,;
    }
}
