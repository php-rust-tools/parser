use structopt::StructOpt;
use phpc::compile;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpc", about = "Compile a PHP script to Rust.")]
struct Args {
    file: String,
}

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