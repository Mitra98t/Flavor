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
    let code = r#"
    let x: array(int) = [2,2,3];
    x[0] = 5;
    print x[0], x[1], x[2];
"#;

    let debug = false;

    if debug {
        println!("Debug prints are ON");
        println!("SOURCE CODE\n{code}\n----\n");
    }

    let mut lexer = Lexer::new(code);
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
    if debug {
        println!("Type Checking\n");
        if let Err(e) = tc.check_program(&nodes) {
            println!("{e}")
        }
        println!("\n----\n");
    }

    let mut interpreter = Interpreter::new();
    interpreter.eval_program(&nodes).unwrap();
}
