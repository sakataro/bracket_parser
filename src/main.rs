use std::{env, process};

extern crate bracket_parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        eprintln!("usage: {} text", args[0]);
        process::exit(1)
    }

    match bracket_parser::parse(&args[1]) {
        Ok(ast) => println!("parsed: {}", ast),
        Err(e) => match e {
            bracket_parser::ParseError::HasNoClosing(at) => {
                eprintln!("not close at: {}", at);
                process::exit(1)
            }
        },
    }
}
