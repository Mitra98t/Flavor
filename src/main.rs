mod lexer;
mod parser;
mod typechecker;
mod types;
use lexer::Lexer;
use parser::Parser;
use typechecker::TypeChecker;

#[allow(unused)]
use crate::types::print_ast;

fn main() {
    let code = r#"

while 5 >2 {
    if true {
        break;
    }
}

fn foo (x: int, y: int) -> int {
    return x + y;
}

foo(4,5);

"#;
    // println!("SOURCE CODE\n{}\n----\n", code);

    let mut lexer = Lexer::new(code);
    lexer.lexe();

    // println!("TOKENS\n");
    // for t in lexer.tokens.clone() {
    //     println!("{:?}", t);
    // }
    // println!("\n----\n");

    let mut parser = Parser::new(lexer.tokens);
    let nodes = parser.parse_program();

    println!("AST\n");
    match &nodes {
        Ok(n) => print_ast(n.to_vec()),
        Err(e) => println!("{}", e),
    }
    println!("\n----\n");

    println!("Type Checking\n");
    let mut tc = TypeChecker::new();
    let nodes = nodes.unwrap();
    if let Err(e) = tc.check_program(&nodes) {
        println!("{}", e)
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typechecker::TypeChecker;
    use crate::types::{ASTNode, Type};

    fn lex_parse_typecheck(code: &str) -> Result<Vec<ASTNode>, String> {
        let mut lexer = Lexer::new(code);
        lexer.lexe();

        let mut parser = Parser::new(lexer.tokens);
        let nodes = parser.parse_program()?;

        let mut tc = TypeChecker::new();
        tc.check_program(&nodes)?;

        Ok(nodes)
    }

    #[test]
    fn test_let_declaration_int() {
        let code = "let x:int = 42;";
        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");

        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                assert_eq!(identifier, "x");
                assert_eq!(var_type, &Some(Type::Int));
                match **expr {
                    ASTNode::NumberLiteral(ref val) => assert_eq!(val, "42"),
                    _ => panic!("Expected NumberLiteral in let expr"),
                }
            }
            _ => panic!("Expected LetDeclaration"),
        }
    }

    #[test]
    fn test_let_declaration_without_type() {
        let code = "let y = 5;";
        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");

        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                assert_eq!(identifier, "y");
                assert_eq!(var_type, &None);
                match **expr {
                    ASTNode::NumberLiteral(ref val) => assert_eq!(val, "5"),
                    _ => panic!("Expected NumberLiteral in let expr"),
                }
            }
            _ => panic!("Expected LetDeclaration"),
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let code = "let x:bool = 10;";
        let mut lexer = Lexer::new(code);
        lexer.lexe();

        let mut parser = Parser::new(lexer.tokens);
        let nodes = parser.parse_program().expect("Should parse");

        let mut tc = TypeChecker::new();
        let result = tc.check_program(&nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_binary_expression_type_check() {
        let code = "let a:int = 1 + 2 * 3;";
        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");

        // Check AST structure roughly
        match &nodes[0] {
            ASTNode::LetDeclaration { var_type, expr, .. } => {
                assert_eq!(var_type, &Some(Type::Int));
                match **expr {
                    ASTNode::BinaryExpression { ref operator, .. } => {
                        assert!(operator == "+" || operator == "*");
                    }
                    _ => panic!("Expected BinaryExpression"),
                }
            }
            _ => panic!("Expected LetDeclaration"),
        }
    }

    #[test]
    fn test_undefined_variable_error() {
        let code = "x;";
        let mut lexer = Lexer::new(code);
        lexer.lexe();

        let mut parser = Parser::new(lexer.tokens);
        let nodes = parser.parse_program().expect("Should parse");

        let mut tc = TypeChecker::new();
        let result = tc.check_program(&nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_expression_statement() {
        let code = "1 + 2;";
        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ASTNode::ExpressionStatement(expr) => match **expr {
                ASTNode::BinaryExpression { ref operator, .. } => {
                    assert_eq!(operator, "+");
                }
                _ => panic!("Expected BinaryExpression inside ExpressionStatement"),
            },
            _ => panic!("Expected ExpressionStatement"),
        }
    }

    #[test]
    fn test_unary_expression() {
        let code = "let x = -5;";
        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");
        match &nodes[0] {
            ASTNode::LetDeclaration { expr, .. } => match **expr {
                ASTNode::UnaryExpression {
                    ref operator,
                    is_postfix,
                    ..
                } => {
                    assert_eq!(operator, "-");
                    assert!(!is_postfix);
                }
                _ => panic!("Expected UnaryExpression"),
            },
            _ => panic!("Expected LetDeclaration"),
        }
    }

    #[test]
    fn test_postfix_expression() {
        let code = "let y = 10; let x = y++;";

        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");
        assert_eq!(nodes.len(), 2);
        match &nodes[1] {
            ASTNode::LetDeclaration { expr, .. } => match **expr {
                ASTNode::UnaryExpression {
                    ref operator,
                    is_postfix,
                    ..
                } => {
                    assert_eq!(operator, "++");
                    assert!(is_postfix);
                }
                _ => panic!("Expected UnaryExpression"),
            },
            _ => panic!("Expected LetDeclaration"),
        }
    }

    #[test]
    fn test_inferred_type_assignment() {
        let code = r#"
let x = 10;
let y:int = 10;

x+y;
        "#;

        let nodes = lex_parse_typecheck(code).expect("Should parse and typecheck");
        assert_eq!(nodes.len(), 3);
        match &nodes[2] {
            ASTNode::ExpressionStatement(n) => match **n {
                ASTNode::BinaryExpression { ref operator, .. } => {
                    assert_eq!(operator, "+");
                }
                _ => panic!("Should be a sum"),
            },
            _ => panic!("Should be expression statement"),
        }
    }
}
