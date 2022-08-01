use std::process::{Command, exit};

use structopt::StructOpt;
use phpc::compile;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpc", about = "Compile a PHP script to Rust.")]
struct Args {
    file: String,
}

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