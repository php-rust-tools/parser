<?php

namespace {
    function null() {
        echo "p\n";
    }
}

namespace bar {
    function null() {
        echo "n";
    }
}

namespace baz {
    use bar;
    use function bar\null as n;
    
    echo n();
    echo bar\null();
    echo \bar\null();
    echo \null();
}
