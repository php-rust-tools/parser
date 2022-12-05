<?php

trait foo {
    // it's okay to have `parent` type
    // since it's not known at this time if
    // `foo` will have a parent.
    public function bar(): parent {
        exit(1);
    }
}
