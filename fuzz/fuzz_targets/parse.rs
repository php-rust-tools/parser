#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match php_parser_rs::parser::parse(data) {
        Ok(ast) => {}
        Err(err) => {}
    }
});
