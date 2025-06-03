mod lexer;
mod parser;
mod typechecker;
mod types;
use lexer::Lexer;
use parser::Parser;
use typechecker::TypeChecker;

fn main() {
    let code = r#"
let x:int = 4;
"#;
    println!("SOURCE CODE\n{}\n----\n", code);

    let mut lexer = Lexer::new(code);
    lexer.lexe();

    let mut parser = Parser::new(lexer.tokens);
    let nodes = parser.parse_program();

    // println!("AST\n");
    // match &nodes {
    //     Ok(n) => print_ast(n.to_vec()),
    //     Err(e) => println!("{}", e),
    // }
    // println!("\n----\n");

    println!("Type Checking\n");
    let mut tc = TypeChecker::new();
    let nodes = nodes.unwrap();
    if let Err(e) = tc.check_program(&nodes) {
        println!("{}", e)
    }
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn tests() {}
}
