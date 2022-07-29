use rust_format::{RustFmt, Formatter};
use structopt::StructOpt;

mod prelude;

#[derive(Debug, StructOpt)]
#[structopt(name = "phpast", about = "Compile a PHP file to Rust.")]
struct Args {
    file: String,

    #[structopt(short, long, help = "Format the generated Rust code.")]
    format: bool,
}

fn main() {
    let args = Args::from_args();

    let prelude = include_str!("./prelude.rs");
    std::fs::write("./build/prelude.rs", prelude);

    let mut source = trunk_rs::compile(args.file).unwrap();
    if args.format {
        source = RustFmt::default().format_str(source).unwrap();
    }

    std::fs::write("./build/input.rs", source).unwrap();
}