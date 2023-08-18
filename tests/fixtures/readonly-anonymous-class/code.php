<?php

$class = new class {
	public readonly ?string $foo;
};

$class = new readonly class {
	public ?string $foo;
};
