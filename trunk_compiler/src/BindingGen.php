<?php

namespace Trunk;

final class BindingGen
{
    private static string $moduleBase;

    private static array $modules = [];

    private static array $functions = [];

    private static array $constructs = [];

    public static function moduleBase(string $base): void
    {
        self::$moduleBase = $base;
    }

    public static function module(string $name): void
    {
        self::$modules[$name] = $name;
    }

    public static function function(string $name, string $module, string $target, int | Range $arity): void
    {
        self::$functions[$name] = [$module, $target, $arity];
    }

    public static function construct(string $keyword, string $module, string $target, int | Range $arity): void
    {
        self::$constructs[$keyword] = [$module, $target, $arity];
    }

    public static function getConstructBinding(string $keyword): ?array
    {
        return self::$constructs[$keyword] ?? null;
    }

    public static function getFunctionBinding(string $name): ?array
    {
        return self::$functions[$name] ?? null;
    }

    public static function resolveModuleName(string $name): string
    {
        if (array_key_exists($name, self::$modules)) {
            return self::$moduleBase . '/' . $name;
        }

        if (str_starts_with($name, '@')) {
            return substr($name, 1);
        }

        return $name;
    }
}