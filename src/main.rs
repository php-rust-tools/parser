use std::io::Result;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    // if --help is passed, or no file is given, print usage in a pretty way and exit
    if args.contains(&String::from("--help")) || args.len() < 2 {
        println!("Usage: php-parser-rs [options] <file>");
        println!("Options:");
        println!("  --help     Print this help message");
        println!("  --silent   Don't print anything");
        println!("  --json     Print as json");
        std::process::exit(0);
    }

    // get file from args or print error and exit
    let file = args.get(1).unwrap();
    let silent = args.contains(&String::from("--silent"));
    let print_json = args.contains(&String::from("--json"));
    let contents = std::fs::read_to_string(file)?;

    match php_parser_rs::parse(&contents) {
        Ok(ast) => {
            // if --silent is passed, don't print anything
            if silent {
                return Ok(());
            }

            // if --json is passed, print as json
            if print_json {
                match serde_json::to_string_pretty(&ast) {
                    Ok(json) => println!("{}", json),
                    Err(error) => {
                        eprintln!("Failed to convert ast to json: {}", error);

                        std::process::exit(1);
                    }
                }
            } else {
                // if --json is not passed, print as text
                println!("{:#?}", ast);
            }
        }
        Err(error) => {
            println!("{}", error.report(&contents, Some(file), true, false)?);

            std::process::exit(1);
        }
    }

    Ok(())
}
