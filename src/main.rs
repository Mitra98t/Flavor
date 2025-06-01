mod lexer;
mod types;
use lexer::Lexer;

fn main() {
    let code = r#"
let
fn
alias

foo
47

: ; = == != ! -> => > < >= <= + - * / % ( ) [ ] { }

let foo = 3;"#;
    let mut lexer = Lexer::new(code);
    lexer.lexe();
    println!("SOURCE CODE\n\n{}\n----", code);

    lexer.tokens.iter().for_each(|tok| {
        println!("{:?}", tok);
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn tests() {}
}
