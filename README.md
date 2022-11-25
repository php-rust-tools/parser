<h3 align="center">
    Trunk
</h3>

<p align="center">
    A collection of modern libraries and tools for working with PHP.
</p>

---

<p align="center">
    <a href="https://discord.gg/49vgTdE6mb">
        <img src="https://img.shields.io/badge/Trunk-%237289DA.svg?style=for-the-badge&logo=discord&logoColor=white">
    </a>
</p>

#### Overview

The Trunk project originally started with me ([Ryan](https://github.com/ryangjchandler)) experimenting with a handwritten recursive-descent parser for PHP written in Rust. The parser evolved very quickly and I found myself already being able to parse relatively complex programs that I had written myself.

Once I had something worth sharing, I created this GitHub repository and some people were interested in what I was doing. Since then, it's become a hobby project for myself and I tend to work on it in my spare time.

The current goal for Trunk is to get a fully functional parser up and running so that it can be used in some meaningful projects, aimed at bringing the speed and performance of modern PHP tooling closer to that of [our](https://deno.land) [cousins](https://bun.sh) in [JavaScript](https://swc.rs/) [land](https://esbuild.github.io).

#### Libraries and Tools

* [Lexer](/trunk_lexer/)
* [Parser](/trunk_parser/)

#### Contributing

All contributions to Trunk are welcome. It's the perfect project for Rust beginners since we don't use many of Rust's complex features and the core concepts in the parser are purposely simple.

If you do wish to contribute, we just ask you to follow a few simple rules.

1. Create a pull request from a **non-main** branch on your fork.
2. Provide a short, but clear, description of your changes.
3. Have fun and don't take it all too seriously!

#### Credits

* [Ryan Chandler](https://github.com/ryangjchandler)
* [All contributors](https://github.com/ryangjchandler/trunk/graphs/contributors)