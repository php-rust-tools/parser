<?php

// PHP considers this a Parse Error, however, we are able to detect this earlier while lexing.
$a = "\u{" ";
