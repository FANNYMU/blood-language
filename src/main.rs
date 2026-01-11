mod ast;
mod interpreter;
mod lexer;
mod parser;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: blood <filename.bd>");
        process::exit(1);
    }

    let filename = &args[1];
    let code = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(code);
    let mut parser = Parser::new(lexer);

    let program = parser.parse_program();

    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.interpret(program) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
