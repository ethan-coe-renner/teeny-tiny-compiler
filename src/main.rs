mod lex;
mod parse;

use std::env;
use std::fs;

fn main() {
    println!("Teeny Tiny Compiler");

    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    let input = fs::read_to_string(filename).expect("Error reading file");

    let lexer = lex::Lexer::new(input);
    let mut parser = parse::Parser::new(lexer);

    parser.program();
    println!("Parsing completed");
}

