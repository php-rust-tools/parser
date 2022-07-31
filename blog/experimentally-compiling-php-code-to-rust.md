# Experimentally compiling PHP code to Rust

## Preface

Some of you might already know that I've been working on a handwritten PHP parser in Rust. The project is called [Trunk (source code on GitHub)](https://github.com/ryangjchandler/trunk).

I've been working on the parser for a couple of weeks at the time of writing this blog post and it's already come a long way. It's able to parse functions, classes, interfaces and much more. It's still miles from being a compliant PHP parser comparable to Nikita's `nikic/php-parser` package, but the journey has been fun so far and it's amazing how many weird things you can find out about a language by parsing it's grammar.

Since the parser is now able to handle some basic programs, I thought it would be worthwhile taking it for a spin to see what the API is like and to look for improvements. Dogfooding, if you want a single word.

My original plan was to work on an experimental runtime and interpreter for the language. That's quite a huge undertaking and there would be very little benefit to a new runtime at this point in time.

Instead I started to think about a compiler for PHP. Something that runs ahead of execution time (AOT). My friend [Tim Morgan](https://twitter.com/timmrgn) is the creator of [Natalie](https://github.com/natalie-lang/natalie), an implementation of the Ruby language that compiles to C++ and then compiles into a native binary. I've contributed to Natalie a little over the last year or so and it's truly inspiring the work that Tim is doing. 

You can probably see where this is going...

## The idea

Inspired by Tim's work on Natalie, I'm going to start an experiment where I take my handwritten PHP parser and try to write a compiler that turns PHP code into Rust and then compiles that Rust code into a native executable using `rustc`.

Let's look at a simple example:

```php
function get_name() {
    return "Ryan";
}

echo get_name();
```

A PHP script like this would eventually compile into some Rust code that looks something like this:

```rust
fn get_name() -> PhpResult<PhpValue> {
    return Ok(PhpValue::from("Ryan"));
}

fn main() -> PhpResult<()>  {
    _php_echo(get_name()?);
}
```

That Rust code could then be stored somewhere and compiled using the Rust compiler, `rustc`.

No more boring introductions. Let's look at some code.

> **Disclaimer**: The following code is incredibly naive and purely for prototyping purposes. There's plenty of rough edges that will need to be ironed out before calling this a "good idea" or "achievable project".

## Parsing the code

I've already got a parser that is capable of understanding the PHP code above. The API is pretty simple, so I decided to just build a small CLI that would accept a file path and send it through the parser to obtain an abstract syntax tree (AST). I'm using the excellent `structopt` crate here for argument parsing.

```rust
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpc", about = "Compile a PHP script to Rust.")]
struct Args {
    file: String,
}

fn main() {
    let args = Args::from_args();
}
```

This is all written inside of a `main.rs` file. I prefer to separate the CLI interface code from the actual logic of the program, so the below code is written inside of `lib.rs` inside of the same folder.

```rust
use trunk_lexer::Lexer;
use trunk_parser::Parser;

pub fn compile(file: String) -> Result<String, CompileError> {
    let contents = match std::fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(_) => return Err(CompileError::FailedToReadFile),
    };

    let mut lexer = Lexer::new(None);
    let tokens = lexer.tokenize(&contents).unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    Ok(String::new())
}

#[derive(Debug)]
pub enum CompileError {
    FailedToReadFile,
}
```

The code takes in a file path and gets a list of tokens using the `Lexer` structure. The tokens are then sent through the parser to retrieve the AST.

> The usage of `unwrap()` isn't great because the program will just panic. A good refactoring opportunity here would be transforming any errors returned from the lexer and parser into `CompileError` values instead.

This function can now be called from `main.rs`.

```rust
use structopt::StructOpt;
use phpc::compile;

#[derive(Debug, Structopt)]
#[structopt(name = "phpc", about = "Compile a PHP script to Rust.")]
struct Args {
    file: String,
}

fn main() {
    let args = Args::from_args();
    let compiled = compile(args.file);

    dbg!(compiled);
}
```

## The `main` conundrum

PHP is a procedural programming language. It's really a scripting language. As such, it doesn't enforce the usage of a `main()` function. The same can be seen in other languages like JavaScript and Python.

Rust on the otherhand requires you to define an `fn main()` somewhere so that the binary knows where to start executing your code. The example PHP code shown further up in this blog post has a single function definition and then an arbitrary statement outside of any known structure.

Think of those arbitrary statements as parts inside of the pseudo `main()` function. The best way to force this structure is by segmenting and partitioning the AST into 2 separate smallers ASTs.

The first will contain any function definitions (for now) and the other will contain all of the rogue statements. That way all of the function definitions can be compiled first and the rest can be compiled inside of an `fn main()` to keep Rust happy.

```rust
use trunk_parser::{Parser, Statement};

pub fn compile(file: String) -> Result<String, CompileError> {
    // ...

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let (fns, main): (Vec<_>, Vec<_>) = ast.iter().partition(|statement| match statement {
        Statement::Function { .. } => true,
        _ => false,
    });

    // ...
}
```

The `partition()` method returns a tuple containing 2 new ASTs. Rust needs some helper understanding the types - the fancy `Vec<_>` syntax just tells the typechecker that the generic type inside of `Vec<T>` is the same as the type inside of the iterator itself (call it a placeholder).

Now it's time for some string building!

## Bare minimal compilation

Compiling our AST into Rust code is just a case of looping over all of the statements and concatenating a bunch of strings. Famous last words, lol.

Let's start with a `compile_function()` method that accepts a `Statement` and `&mut String`.

```rust
fn compile_function(function: &Statement, source: &mut String) -> Result<(), CompileError> {
    let (name, params, body) = match function {
        Statement::Function { name, params, body, .. } => (name, params, body),
        _ => unreachable!(),
    };

    source.push_str("fn ");
    source.push_str(&name.name);
    source.push('(');

    for param in params {
        source.push_str(match &param.name {
            Expression::Variable(n) => &n,
            _ => unreachable!(),
        });
        source.push_str(": PhpValue, ");
    }

    source.push_str(") -> PhpValue {");

    for statement in body {
        compile_statement(statement, source)?;
    }

    source.push('}');

    Ok(())
}
```

This function is going to extract some values from the `Statement` and return a tuple that we can unpack. There's some boilerplate code to output the `fn` keyword along with it's name.

The function then needs a list of parameters. We don't really care about parameter types right now because all of our values are going to be represented by a single `PhpValue` type in the Rust code anyway.

It also needs a return type which will also be a `PhpValue`. It's then just a case of looping over all of the statements inside of the function and compiling those before closing off the function itself.

Pretty simple so far... let's implement a minimal `compile_statement()` function too.

```rust
fn compile_statement(statement: &Statement, source: &mut String) -> Result<(), CompileError> {
    match statement {
        Statement::Return { value } => {
            source.push_str("return");
            if let Some(value) = value {
                source.push(' ');
                source.push_str(&compile_expression(value)?);
            } else {
                todo!();
            }
            source.push(';');
        },
        _ => todo!(),
    };

    Ok(())
}
```

Our example `get_name()` function only contains a `return` statement so we'll just implement that for now. It's more string building here, with the addition of a new `compile_expression()` function too.

You might notice that we're actually pushing the result of `compile_expression()` onto the string instead of passing through a mutable reference to `source`. This is important since we might want to compile an expression arbitrarily without modifying the source code.

We've got another function to write so let's do that real quick.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        Expression::ConstantString(value) => {
            format!(r#"PhpValue::from("{}")"#, value)
        }
        _ => todo!(),
    };

    Ok(result)
}
```

Still implementing the bare minimum here. The `get_name()` function returns a constant string which will be able to directly convert into a `PhpValue` type with some traits later on. The `format!()` macro returns a `String` so we can just keep it simple and avoid any temporary variables.

Time to wire it all up inside of the `compile()` function.

```rust
fn compile(file: String) -> Result<String, CompileError> {
    // ...

    let mut source = String::new();

    for function in fns {
        compile_function(function, &mut source)?;
    }

    Ok(source)
}
```

Create an empty `String`, loop through the function statements, compile them, return the generated source code.

Running the example code through the program now will dump the generated Rust code into the terminal. It looks like this:

```sh
$ ~ cargo run --bin phpc -- ./phpc/samples/blog.php
[phpc/src/main.rs:14] compiled = Ok(
    "fn get_name() -> PhpValue {return PhpValue::from(\"Ryan\");}",
)
```

Our function has been generated with a rather ugly return statement inside! Visible progress - love it. Time to get the `main()` function.

## Compiling the entrypoint

Before looping through the main statements, we need to add the boilerplate code. This can all go inside of the `compile()` function for now.

```rust
pub fn compile(file: String) -> Result<String, CompileError> {
    // ...

    source.push_str("fn main() {");

    for statement in main {
        compile_statement(statement, &mut source)?;
    }

    source.push('}');

    Ok(source)
}
```

Running this code will cause some panics since we've not implemented all of our statements. We need to add in support for `echo`.

```rust
fn compile_statement(statement: &Statement, source: &mut String) -> Result<(), CompileError> {
    match statement {
        // ...
        Statement::Echo { values } => {
            for value in values {
                source.push_str("_php_echo(");
                source.push_str(&compile_expression(value)?);
                source.push_str(");");
            }
        },
        _ => todo!(),
    };

    Ok(())
}
```

Trying to compile the example code again triggers another panic, this time it's the new call to `compile_expression()`. The compiler doesn't know how to generate Rust code for calling PHP functions.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Call(target, args) => {
            let mut buffer = String::new();

            buffer.push_str(&compile_expression(target)?);
            buffer.push('(');

            for arg in args {
                buffer.push_str(&compile_expression(arg)?);
                buffer.push_str(", ");
            }

            buffer.push(')');
            buffer
        },
        Expression::Identifier(i) => i.to_string(),
        _ => todo!(),
    };

    Ok(result)
}
```

The code above should be pretty self-explanatory at this point. A `Call` expression has a target and list of arguments. The target is also an `Expression` so needs to be compiled, as well as the arguments.

In the example script, the target is just an identifier so the function needs to know how to compile that as well. Luckily it just needs to output the name of the identifer as a literal string. The `.to_string()` method call is required to generate a non-referential string.

Running the example script through the compiler one more time generates the following Rust code.

```sh
$ ~ cargo run --bin phpc -- ./phpc/samples/blog.php
[phpc/src/main.rs:14] compiled = Ok(
    "fn get_name() -> PhpValue {return PhpValue::from(\"Ryan\");}fn main() {_php_echo(get_name());}",
)
```

A little harder to read now but the `main()` function exists and has the expected code inside of it. Woop woop!

## The runtime

The compiler is pretty smart now. It can take an incredibly complex PHP script and generate valid Rust code. Or can it?

The answer is of course **no**. The Rust code is referencing a weird `PhpValue` type that doesn't actually exist in the generated code. There are a few potential ways to solve this but I'm going to go with the simplest and fastest way - a "runtime" file.

This runtime file will contain all of the code needed to actually run PHP code. It's going to be house the `PhpValue` type as well as any internal functions that handle PHP concepts, such as `_php_echo()`.

Rust makes this really simple as well with the `include_str!()` macro. This macro is evaluated at compile time and takes in a file path. The contents of that file are then embedded into the program and can be assigned to a variable.

Adding that code into the `compile()` function looks like this:

```rust
pub fn compile(file: String) -> Result<String, CompileError> {
    // ...

    let mut source = String::new();
    source.push_str(include_str!("./runtime.rs"));

    // ...

    Ok(source)
}
```

The top of the generated code will now include everything inside of the `runtime.rs` file.

This file can now house the runtime information such as `PhpValue` and `_php_echo()`.

```rust
use std::fmt::Display;

enum PhpValue {
    String(String),
}

impl From<&str> for PhpValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl Display for PhpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => write!(f, "{}", string),
            _ => todo!(),
        }
    }
}

fn _php_echo(value: PhpValue) {
    print!("{value}");
}
```

The only type of PHP value that the example script uses is a `string`, so the `PhpValue` can just have a single variant.

I want the `PhpValue` type to be smart and understand conversions from native Rust types. That's where the generic `From` trait comes in. It tells the Rust compiler how to convert a `&str` into a `PhpValue`.

`PhpValue` also implements the `Display` trait which will allow the struct to be formatted with Rust's first party macros such as `write!()`, `print!()` and `println!()`. An added bonus of implementing `Display` is that you get a `.to_string()` method on the implementor for free.

The `_php_echo()` function takes in a value and just prints it out using `print!()`.

## Compiling with `rustc`

The compiler has just about everything it needs to compile the example PHP script. The last step is teaching the CLI how to take the compiled code and write it to disk and then send that file through Rust's compiler, `rustc`.

```rust
fn main() {
    let args = Args::from_args();
    let compiled = compile(args.file.clone()).unwrap();

    let path = std::path::Path::new(&args.file);
    let temp = std::env::temp_dir();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let file_path = format!("{}{}.rs", temp.to_str().unwrap(), &file_stem);

    std::fs::write(&file_path, compiled).unwrap();

    println!("Compiled code written to {}...", &file_path);
    println!("Generating binary...");

    std::process::Command::new("rustc")
        .args([file_path, "-o".to_string(), file_stem.to_string()])
        .output()
        .expect("Failed to compile with rustc");
}
```

The file name is converted into a `Path` instance. This gives us a handy `file_stem()` method that returns the file name without the extension. A full file path is then generated using a temporary directory, the file name and `.rs` file extension.

All of the compiled code is then written to that temporary file path and printed in the terminal for debugging purposes.

A new `Command` is then created targeting the global `rustc` binary. The `file_path` is provided as the input file and the output is set using the `-o` flag followed by the name of the binary. For convenience the binary is named after the original input file to the compiler.

The `Command` then executes and will panic if it fails to read any output. This is pretty poor since even if `rustc` fails to compile because of bad Rust code, the `phpc` program will be successful. It'll do for now though.

## The moment you've all been waiting for

It's time to compile the example script...

```sh
$ ~ cargo run --bin phpc -- ./phpc/samples/blog.php
Compiled code written to /var/folders/52/3xkzr5_s1zbczrn85z2kqfv40000gn/T/blog.rs...
Generating binary...
```

And it works as expected. In the current working directory is a new `blog` file which can be executed in the terminal.

```sh
$ ~ ./blog
Ryan%
```

Absolute sorcery! Four lines of incredibly simple PHP code were parsed with a handwritten parser, compiled into a Rust file with a novel compiler and compiled into a native binary using Rust's own toolchain.

Please, please, please, remember that this is just a prototype and proof of concept right now. I'm most definitely going to continue the development of this alongside the parser, but right now it's still a proof of concept!

There's plenty of hurdles to jump before it becomes remotely useful for anybody. Already my mind is thinking "How will it handle `require` and `include`?", "What about conditionally defining functions?", "How do you even represent a PHP interface with Rust code!?" and "I forgot about closures...".

I'm going to continue writing about my journey developing Trunk, no matter the outcome. Regardless of the success with the compiler, the parser work will continue and I'm sure we'll find lots of other interesting projects to experiment with too.

If you made it this far without drifting off and getting some sleep, thank you.

Until next time.