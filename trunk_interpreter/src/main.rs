use cmder::Program;

mod cmd;
mod engine;

fn main() {
    let mut program = Program::new();

    program
        .bin_name("trunk")
        .author("Ryan Chandler <support@ryangjchandler.co.uk>")
        .description("An alternative interpreter and runtime for PHP.");

    let run_cmd = program.subcommand("run");

    run_cmd
        .description("Run a PHP file through Trunk.")
        .alias("r")
        .argument("file", "The file you'd like to run.")
        .action(cmd::run);

    program.parse();
}
