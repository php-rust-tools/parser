<?php

/**
 * A function with a lot of comments.
 */
function foo
    // a single line comment on all parameters
    (
        // a single line comment on the first parameter
        string /* a comment between `string` and `$a` */ $a,
        # a hash comment on the second parameter
        string /* a comment between `string` and `$b` */ $b,
        /* a multi-line comment on the third parameter */
        string /* a comment between `string` and `$c` */ $c,
        /** a document comment on the fourth parameter */
        string /* a comment between `string` and `$d` */ $d,
    )
{}
