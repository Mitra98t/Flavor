mod error;
mod interpreter;
mod lexer;
mod parser;
mod typechecker;
mod types;

use error::FlavorError;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use typechecker::TypeChecker;

#[allow(unused)]
use crate::types::print_ast;

fn report_and_exit(error: FlavorError, source: &str) -> ! {
    eprintln!("{}", error.render(source));
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let code = std::fs::read_to_string(filename).expect("Failed to read source file");

    let debug = false;

    if debug {
        println!("Debug prints are ON");
        println!("SOURCE CODE\n{code}\n----\n");
    }

    let mut lexer = Lexer::new(&code);
    lexer.lexe();

    if debug {
        println!("TOKENS\n");
        for t in lexer.tokens.clone() {
            println!("{t:?}");
        }
        println!("\n----\n");
    }

    let mut parser = Parser::new(lexer.tokens);
    let nodes = match parser.parse_program() {
        Ok(nodes) => nodes,
        Err(err) => report_and_exit(err, &code),
    };

    if debug {
        println!("AST\n");
        print_ast(nodes.clone());
        println!("\n----\n");
    }

    let mut tc = TypeChecker::new();
    if let Err(err) = tc.check_program(&nodes) {
        report_and_exit(err, &code);
    }

    if debug {
        println!("Type Checking\n\n----\n");
    }

    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.eval_program(&nodes) {
        report_and_exit(err, &code);
    }
}
