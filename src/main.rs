mod lex;
mod parse;
mod emit;

use std::env;
use std::fs;

fn main() {
    println!("Teeny Tiny Compiler");

    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    let input = fs::read_to_string(filename).expect("Error reading file");

    let lexer = lex::Lexer::new(input);
    let emitter = emit::Emitter::new("out.c");
    let mut parser = parse::Parser::new(lexer, emitter);

    parser.program();
    println!("Compiling completed.");
}

