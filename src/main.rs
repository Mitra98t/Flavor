mod interpreter;
mod lexer;
mod parser;
mod typechecker;
mod types;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use typechecker::TypeChecker;

#[allow(unused)]
use crate::types::print_ast;

fn main() {
    // read from file given in input
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
    let nodes = parser.parse_program();

    if debug {
        println!("AST\n");
        match &nodes {
            Ok(n) => print_ast(n.to_vec()),
            Err(e) => println!("{e}"),
        }
        println!("\n----\n");
    }

    let mut tc = TypeChecker::new();
    let nodes = nodes.unwrap();
    let typecheck_result = tc.check_program(&nodes);
    if debug {
        println!("Type Checking\n");
        if let Err(e) = &typecheck_result {
            println!("{e}")
        }
        println!("\n----\n");
    }
    typecheck_result.unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.eval_program(&nodes).unwrap();
}
