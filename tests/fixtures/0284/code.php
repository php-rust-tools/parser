<?php 

//                                                    |- TODO: static in constant expression is not allowed.
//                                                    |
//                                                    v
#[foo(self::class), bar(new self(), new parent(), new static())]
class a {

}
