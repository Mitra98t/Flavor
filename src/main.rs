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
    let x: int = 10;
    while ( x > 0 ) {
        print(x);
        x--;
        if (x == 5) {
            break;
        } else {
            print("x is not 5");
        }
    }
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
    let typecheck_result = tc.check_program(&nodes);

    if debug {
        println!("Type Checking\n");
        if let Err(e) = &typecheck_result {
            println!("{e}")
        }
        println!("\n----\n");
    }

    let mut interpreter = Interpreter::new();
    interpreter.eval_program(&nodes).unwrap();
}
