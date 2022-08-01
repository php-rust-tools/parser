# Compiling PHP Conditional Statements to Rust

Welcome back to the series. It's time to take another step into compilation land and look at compiling PHP's conditional statements (`if` statements) into Rust code.

The goal for this post will be compiling the following code:

```php
$guess = readline("Guess a number between 1 and 3: ");
$number = rand(1, 3);

if ($guess == $number) {
    echo "You guessed the number correctly, well done!";
} else {
    echo "The correct answer is " . $number . ". Better luck next time!";
}
```

Before we start writing some Rust, let's analyze the code and look at the things we'll need to implement.

At the very top of the script we've got some variable assignments. This is a type of expression so we'll need to add some new code to the `compile_expression()` function.

On the right-hand side of those assignments we're calling some native / first-party PHP functions. These don't exist in our runtime at the moment, so we'll need to implement those in Rust land as part of our `runtime.rs` file.

We then reach the conditional statements. We'll need to handle the compilation of the structure itself, along with the expressions used inside of blocks.

The condition in our `if` statement uses the `==` operator which is referred to as an **infix** operator. The `compile_expression()` will need to be updated to handle this new type of expression as well. We'll also need to keep in mind that Rust doesn't have any concept of loose or strict comparisons, instead we'll need to implement thing logic ourself.

Let start by supporting the assignment expression and writing our own implementations of `readline()` and `rand()`.

## Assignment expressions

An assignment expression in the Rust code will be represented with a `let` statement. PHP variables are all mutable but Rust lets us redeclare and rebind a variable after it's original definition with another `let` statement. Here's an example.

```rust
let foo = 1;
let foo = 2;
```

The second binding will replace the original without needing to make the original assignment mutable. Let's add this code to `compile_expression()`.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Assign(target, value) => {
            format!("let {} = {};", compile_expression(target)?, compile_expression(value)?)
        },
        _ => todo!(),
    };

    Ok(result)
}
```

If we try to run this code, there will actually be an unimplemented / todo panic earlier on in the code. Our parser actually stores random expressions like an assignment inside of a statement, so we need to tell `compile_statement()` to send our expression statements through the `compile_expression()` function.

```rust
fn compile_statement(statement: &Statement, source: &mut String) -> Result<(), CompileError> {
    match statement {
        // ...
        Statement::Expression { expr } => {
            source.push_str(&compile_expression(expr)?);
        },
        _ => todo!(),
    };

    Ok(())
}
```

The next thing to do is add a new `Int` type to the `PhpValue` enumeration and compile integer expressions. This is so we can eventually call the `rand()` function.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Int(i) => format!("PhpValue::from({})", i),
        _ => todo!(),
    };

    Ok(result)
}
```

And updating our `PhpValue` enumeration to support creation from an `i64`.

```rust
enum PhpValue {
    String(String),
    Int(i64),
}

impl From<i64> for PhpValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}
```

Let's write some native PHP functions in Rust. We'll start with `readline()`.

This function accepts an optional string which will be printed before asking for user input. We'll make this argument required for now since our compiler doesn't know how to handle optional arguments just yet.

```rust
pub fn readline(prompt: PhpValue) -> PhpValue {
    print!("{}", prompt);

    std::io::stdout().flush().unwrap();

    let mut result = String::new();
    std::io::stdin().lock().read_line(&mut result).unwrap();

    PhpValue::from(result.trim_end())
}
```

We flush `stdout()` so that any previous `print!()` calls reach the terminal before we lock it for input. We then read in a line of text from the terminal and store it inside of the `result` variable.

Rust's `read_line()` method will also include the `\n` character at the end of the string so using a `.trim_end()` call will tidy that up.

Time for the `rand()` function. This is going to require a third-party crate since I don't particularly want to write my own PRNG. We'll be using the defacto `rand` crate.

```rust
use rand::Rng;

fn rand(from: PhpValue, to: PhpValue) -> PhpValue {
    let from: i64 = from.into();
    let to: i64 = to.into();

    let mut rng = rand::thread_rng();

    PhpValue::from(rng.gen_range(from..to))
}
```

For type conversions between native Rust types and `PhpValue`, we'll start to implement `Into<T>` traits. Rust will call the appropriate method based on the inferred type or provided type of the target, in this case the `from` and `to` values are both `i64` so it will call the `Into<i64>` method.

```rust
impl Into<i64> for PhpValue {
    fn into(self) -> i64 {
        match self {
            Self::Int(i) => i,
            _ => todo!(),
        }
    }
}
```

## Compiling `if` statements

This might sound a complex task but since we're compiling from one language to another, we can actually take advantage of Rust's own conditional statements. We'll just be translating one syntax to another.

Updating `compile_statement()` to support conditionals is quite simple:

```rust
fn compile_statement(statement: &Statement, source: &mut String) -> Result<(), CompileError> {
    match statement {
        // ...
        Statement::If { condition, then, else_ifs, r#else } => {
            source.push_str("if ");
            source.push_str(&compile_expression(condition)?);
            source.push('{');

            for statement in then {
                compile_statement(statement, source)?;
            }

            source.push('}');

            if let Some(r#else) = r#else {
                source.push_str("else {");
                for statement in r#else {
                    compile_statement(statement, source)?;
                }
                source.push('}');
            }
        },
        _ => todo!(),
    };

    Ok(())
}
```

It doesn't support any `elseif` conditions at the moment since those don't exist in our sample code. For now it compiles the initial `if` statement and checks to see if there is a valid `else` statement at the end. If there is it compiles that too.

We'll also need to support equality checks inside of `compile_expression()`. There's a couple of ways to do this.

1. Manually implement `PartialEq` on the `PhpValue` enumeration and perform the equality comparisons there.
2. Write our own `.eq()` and `.identical()` methods since PHP has some type juggling rules that would be easier to implement here.

I'm going to go with option 2 here since I think there will be more long-term flexibility when compared to Rust's own `PartialEq` trait. Right now the compiler only needs to know about loose comparisons so we'll only implement the `eq()` method.

```rust
impl PhpValue {
    pub fn eq(&self, other: Self) -> bool {
        match (self, &other) {
            (Self::Int(a), Self::String(b)) | (Self::String(b), Self::Int(a)) => match b.parse::<i64>() {
                Ok(b) => *a == b,
                _ => false,
            },
            _ => todo!(),
        }
    }
}
```

The result of `readline()` should be a string so the compiler only needs to support loose comparisons between `Int` and `String` right now. Rust doesn't let you do this natively so the first step is to try and parse an `i64` from the given `String`.

If that is successful, the result of the function will be an equality check between the `a` and `b`. If it fails it means the `String` couldn't be converted into an `i64` and it's impossible for the values to be equal.

Now for the expression compilation itself.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Infix(lhs, op, rhs) => {
            let lhs = compile_expression(lhs)?;
            let rhs = compile_expression(rhs)?;

            match op {
                InfixOp::Equals => format!("{}.eq({})", lhs, rhs),
                _ => todo!(),
            }
        },
        Expression::Variable(var) => var.to_string(),
        _ => todo!(),
    };

    Ok(result)
}
```

The compiler didn't know how to handle variables either so that has been added too. 

The last type of expression the compiler needs to understand is string concatenation. This is another type of infix operation so it's a case of adding another pattern to the `match` expression.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Infix(lhs, op, rhs) => {
            let lhs = compile_expression(lhs)?;
            let rhs = compile_expression(rhs)?;

            match op {
                InfixOp::Equals => format!("{}.eq({})", lhs, rhs),
                InfixOp::Concat => format!("_php_concat({}, {})", lhs, rhs),
                _ => todo!(),
            }
        },
        // ...
        _ => todo!(),
    };

    Ok(result)
}
```

Instead of mutating existing `PhpValue` values the runtime will create an entirely new one from 2 separate `PhpValue` arguments. This function needs to be written in the `runtime.rs` file alongside our `_php_echo()` function.

```rust
fn _php_concat(left: PhpValue, right: PhpValue) -> PhpValue {
    format!("{}{}", left, right).into()
}
```

In the previous post we implemented the `Display` trait for `PhpValue` which allows us to natively use the enumerations inside of Rust's first-party formatting macros such as `format!()`, `print!()` and `println!()`.

With all of that done, it's time to compile the file! And it doesn't work. 

## Using dependencies

If you haven't read the first blog post, the way this compiler works is by essentially concatenating the compiled PHP code with a `runtime.rs` file which is written inside of the `phpc` crate. That file is then stored inside of a temporary directory and compiled using `rustc` directly.

The problem with this approach is that we can't use any external dependencies inside of the `runtime.rs` file because they're not going to be linked against during compilation. 

One potential solution to this problem is generating a static object file for the runtime and linking against that when compiling the PHP code. As the dependency list grows though the number of libraries that would need to be compiled would grow quite quickly.

I'm instead going to go down the route of ditching `rustc` and using `cargo` to build the project instead. The benefit here is that we can let `cargo` do all of the heavy lifting instead and run away from the `rustc` API.

I won't go over each step individually but will just paste the new `main` function code here.

```rs
fn main() {
    let args = Args::from_args();

    println!("> Compiling PHP script...");

    let compiled = compile(args.file.clone()).unwrap();

    let path = std::path::Path::new(&args.file);
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    
    let temp_dir = std::env::temp_dir();
    let temp_path = format!("{}{}", temp_dir.to_str().unwrap(), Uuid::new_v4());

    println!("> Initialising Cargo project in {}...", &temp_path);

    std::fs::create_dir(&temp_path).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["init", ".", "--name", &file_stem])
        .current_dir(&temp_path);

    match cmd.output() {
        Ok(o) => {
            print!("{}", String::from_utf8_lossy(&o.stdout));
        },
        Err(e) => {
            eprintln!("Failed to generate Cargo project. Error: {:?}", e);
            exit(1);
        },
    };

    let cargo_stub = include_str!("../../phpc_runtime/Cargo.toml").replace("phpc_runtime", file_stem);

    println!("> Modifying Cargo configuration...");

    match std::fs::write(format!("{}/Cargo.toml", &temp_path), cargo_stub) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to modify Cargo configuration. Error: {:?}", e);
            exit(1);
        },
    };

    let runtime_stub = include_str!("../../phpc_runtime/src/lib.rs");
    
    println!("> Writing runtime module...");

    match std::fs::write(format!("{}/src/runtime.rs", &temp_path), runtime_stub) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to write runtime library. Error: {:?}", e);
            exit(1);
        }
    };

    println!("> Writing compiled PHP code...");

    match std::fs::write(format!("{}/src/main.rs", &temp_path), compiled) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to write compiled PHP code. Error: {:?}", e);
            exit(1);
        },
    };

    println!("> Compiling project with Cargo...");

    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--release"])
        .current_dir(&temp_path);

    match cmd.output() {
        Ok(o) => {
            if o.status.success() {
                print!("{}", String::from_utf8_lossy(&o.stdout));
            } else {
                print!("{}", String::from_utf8_lossy(&o.stderr));
            }
        },
        Err(e) => {
            eprintln!("Failed to compile project with Cargo. Error: {:?}", e);
            exit(1);
        },
    };
}
```

The error handling of commands could be a lot tidier if there was a wrapper around the `Command` API, but this will do for now. The `Cargo.toml` file for the PHP project is taken from a new `phpc_runtime` crate and modified slightly.

Dependencies are now compiled into our project correctly, but there's still a problem. The compiled Rust code isn't memory safe!

The problematic code is coming from our equality expression. We're moving the value out of the current block into the `.eq()` function which means we can no longer reference it in the main flow of execution. A hacky fix is just to clone the value before we send it through, that way the original value isn't actually being used and a freshly allocated one is instead.

```rust
fn compile_expression(expression: &Expression) -> Result<String, CompileError> {
    let result = match expression {
        // ...
        Expression::Infix(lhs, op, rhs) => {
            let lhs = compile_expression(lhs)?;
            let rhs = compile_expression(rhs)?;

            match op {
                InfixOp::Equals => format!("{}.eq(({}).clone())", lhs, rhs),
                InfixOp::Concat => format!("_php_concat({}, {})", lhs, rhs),
                _ => todo!(),
            }
        },
        // ...
        _ => todo!(),
    };

    Ok(result)
}
```

> A better way of doing this is probably with a smart pointer or even a garbage collector. Since we're still prototyping this code a `.clone()` is fine and won't hurt anybody.

The project successfully compiles but the executable hasn't been moved into the current working directory. A bit of extra logic at the end of the `main()` function should sort this.

```rust
fn main() {
    // ...

    let executable_path = format!("{}/target/release/{}", &temp_path, &file_stem);

    match std::fs::copy(executable_path, format!("./{}", &file_stem)) {
        Ok(_) => {
            println!("> Executable copied.");
        },
        Err(e) => {
            eprintln!("Failed to copy executable file. Error: {:?}", e);
            exit(1);
        },
    };
}
```

Compiling the project now will create a new `guess-a-number` file in the current directory. Executing that file and trying to guess a number results in this:

```sh
$ ~ ./guess-a-number
Guess a number between 1 and 3: 1
You guessed the number correctly, well done!%
```

Here's a list of the things that accomplished:

* Compile variable assignments.
* Write a native PHP function in Rust.
* Compile `if` and `else` statements.
* Migrate project compilation to Cargo to allow use of external crates.

All in all this was a pretty successful article. Again, if you made it to the end then thank you.