mod lexer;
mod parser;
mod types;
use lexer::Lexer;
use parser::Parser;
use types::print_ast;

fn main() {
    let code = r#"
x++;
--x;

3+4+5*6;

foo[2+6];
"#;
    println!("SOURCE CODE\n{}\n----\n", code);

    let mut lexer = Lexer::new(code);
    lexer.lexe();

    let mut parser = Parser::new(lexer.tokens);
    let nodes = parser.parse_program();

    println!("AST\n");
    match nodes {
        Ok(n) => print_ast(n),
        Err(e) => println!("{}", e),
    }
    println!("\n----\n");
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn tests() {}
}
