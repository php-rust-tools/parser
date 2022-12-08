<?php

static fn (Foo &...$s): Bar => Bar::from($s);
static fn (Foo ...$s): Bar => Bar::from($s);
static fn (Foo &$s): Bar => Bar::from($s);
static fn (Foo $s): Bar => Bar::from($s);
static function (Foo &...$s): Bar { return Bar::from($s); };
static function (Foo ...$s): Bar { return Bar::from($s); };
static function (Foo &$s): Bar { return Bar::from($s); };
static function (Foo $s): Bar { return Bar::from($s); };
static fn (Foo&Bar&Baz &...$s): Bar => Bar::from($s);
static fn (Foo&Bar&Baz ...$s): Bar => Bar::from($s);
static fn (Foo&Bar&Baz &$s): Bar => Bar::from($s);
static fn (Foo&Bar&Baz $s): Bar => Bar::from($s);
static function (Foo&Bar&Baz &...$s): Bar { return Bar::from($s); };
static function (Foo&Bar&Baz ...$s): Bar { return Bar::from($s); };
static function (Foo&Bar&Baz &$s): Bar { return Bar::from($s); };
static function (Foo&Bar&Baz $s): Bar { return Bar::from($s); };
static fn (Foo|Bar|Baz &...$s): Bar => Bar::from($s);
static fn (Foo|Bar|Baz ...$s): Bar => Bar::from($s);
static fn (Foo|Bar|Baz &$s): Bar => Bar::from($s);
static fn (Foo|Bar|Baz $s): Bar => Bar::from($s);
static function (Foo|Bar|Baz &...$s): Bar { return Bar::from($s); };
static function (Foo|Bar|Baz ...$s): Bar { return Bar::from($s); };
static function (Foo|Bar|Baz &$s): Bar { return Bar::from($s); };
static function (Foo|Bar|Baz $s): Bar { return Bar::from($s); };
static fn (Foo|(Bar&Baz) &...$s): Bar => Bar::from($s);
static fn (Foo|(Bar&Baz) ...$s): Bar => Bar::from($s);
static fn (Foo|(Bar&Baz) &$s): Bar => Bar::from($s);
static fn (Foo|(Bar&Baz) $s): Bar => Bar::from($s);
static function (Foo|(Bar&Baz) &...$s): Bar { return Bar::from($s); };
static function (Foo|(Bar&Baz) ...$s): Bar { return Bar::from($s); };
static function (Foo|(Bar&Baz) &$s): Bar { return Bar::from($s); };
static function (Foo|(Bar&Baz) $s): Bar { return Bar::from($s); };
static fn (Foo&(Bar|Baz) &...$s): Bar => Bar::from($s);
static fn (Foo&(Bar|Baz) ...$s): Bar => Bar::from($s);
static fn (Foo&(Bar|Baz) &$s): Bar => Bar::from($s);
static fn (Foo&(Bar|Baz) $s): Bar => Bar::from($s);
static function (Foo&(Bar|Baz) &...$s): Bar { return Bar::from($s); };
static function (Foo&(Bar|Baz) ...$s): Bar { return Bar::from($s); };
static function (Foo&(Bar|Baz) &$s): Bar { return Bar::from($s); };
static function (Foo&(Bar|Baz) $s): Bar { return Bar::from($s); };

fn (Foo &...$s): Bar => Bar::from($s);
fn (Foo ...$s): Bar => Bar::from($s);
fn (Foo &$s): Bar => Bar::from($s);
fn (Foo $s): Bar => Bar::from($s);
function (Foo &...$s): Bar { return Bar::from($s); };
function (Foo ...$s): Bar { return Bar::from($s); };
function (Foo &$s): Bar { return Bar::from($s); };
function (Foo $s): Bar { return Bar::from($s); };
fn (Foo&Bar&Baz &...$s): Bar => Bar::from($s);
fn (Foo&Bar&Baz ...$s): Bar => Bar::from($s);
fn (Foo&Bar&Baz &$s): Bar => Bar::from($s);
fn (Foo&Bar&Baz $s): Bar => Bar::from($s);
function (Foo&Bar&Baz &...$s): Bar { return Bar::from($s); };
function (Foo&Bar&Baz ...$s): Bar { return Bar::from($s); };
function (Foo&Bar&Baz &$s): Bar { return Bar::from($s); };
function (Foo&Bar&Baz $s): Bar { return Bar::from($s); };
fn (Foo|Bar|Baz &...$s): Bar => Bar::from($s);
fn (Foo|Bar|Baz ...$s): Bar => Bar::from($s);
fn (Foo|Bar|Baz &$s): Bar => Bar::from($s);
fn (Foo|Bar|Baz $s): Bar => Bar::from($s);
function (Foo|Bar|Baz &...$s): Bar { return Bar::from($s); };
function (Foo|Bar|Baz ...$s): Bar { return Bar::from($s); };
function (Foo|Bar|Baz &$s): Bar { return Bar::from($s); };
function (Foo|Bar|Baz $s): Bar { return Bar::from($s); };
fn (Foo|(Bar&Baz) &...$s): Bar => Bar::from($s);
fn (Foo|(Bar&Baz) ...$s): Bar => Bar::from($s);
fn (Foo|(Bar&Baz) &$s): Bar => Bar::from($s);
fn (Foo|(Bar&Baz) $s): Bar => Bar::from($s);
function (Foo|(Bar&Baz) &...$s): Bar { return Bar::from($s); };
function (Foo|(Bar&Baz) ...$s): Bar { return Bar::from($s); };
function (Foo|(Bar&Baz) &$s): Bar { return Bar::from($s); };
function (Foo|(Bar&Baz) $s): Bar { return Bar::from($s); };
fn (Foo&(Bar|Baz) &...$s): Bar => Bar::from($s);
fn (Foo&(Bar|Baz) ...$s): Bar => Bar::from($s);
fn (Foo&(Bar|Baz) &$s): Bar => Bar::from($s);
fn (Foo&(Bar|Baz) $s): Bar => Bar::from($s);
function (Foo&(Bar|Baz) &...$s): Bar { return Bar::from($s); };
function (Foo&(Bar|Baz) ...$s): Bar { return Bar::from($s); };
function (Foo&(Bar|Baz) &$s): Bar { return Bar::from($s); };
function (Foo&(Bar|Baz) $s): Bar { return Bar::from($s); };

function foo(Foo &...$s): Bar { return Bar::from($s); }
function foo(Foo ...$s): Bar { return Bar::from($s); }
function foo(Foo &$s): Bar { return Bar::from($s); }
function foo(Foo $s): Bar { return Bar::from($s); }
function foo(Foo&Bar&Baz &...$s): Bar { return Bar::from($s); }
function foo(Foo&Bar&Baz ...$s): Bar { return Bar::from($s); }
function foo(Foo&Bar&Baz &$s): Bar { return Bar::from($s); }
function foo(Foo&Bar&Baz $s): Bar { return Bar::from($s); }
function foo(Foo|Bar|Baz &...$s): Bar { return Bar::from($s); }
function foo(Foo|Bar|Baz ...$s): Bar { return Bar::from($s); }
function foo(Foo|Bar|Baz &$s): Bar { return Bar::from($s); }
function foo(Foo|Bar|Baz $s): Bar { return Bar::from($s); }
function foo(Foo|(Bar&Baz) &...$s): Bar { return Bar::from($s); }
function foo(Foo|(Bar&Baz) ...$s): Bar { return Bar::from($s); }
function foo(Foo|(Bar&Baz) &$s): Bar { return Bar::from($s); }
function foo(Foo|(Bar&Baz) $s): Bar { return Bar::from($s); }
function foo(Foo&(Bar|Baz) &...$s): Bar { return Bar::from($s); }
function foo(Foo&(Bar|Baz) ...$s): Bar { return Bar::from($s); }
function foo(Foo&(Bar|Baz) &$s): Bar { return Bar::from($s); }
function foo(Foo&(Bar|Baz) $s): Bar { return Bar::from($s); }
