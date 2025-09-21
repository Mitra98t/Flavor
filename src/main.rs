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
    //check file extension
    if !filename.ends_with(".flv") {
        eprintln!("Error: Source file must have a .flv extension");
        std::process::exit(1);
    }
    let code = std::fs::read_to_string(filename).expect("Failed to read source file");

    let debug = false;

    if debug {
        println!("Debug prints are ON");
        println!("SOURCE CODE\n{code}\n----\n");
    }

    let mut lexer = Lexer::new(&code);
    let lex_res = lexer.lexe();
    if let Err(err) = lex_res {
        report_and_exit(err, &code);
    }

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
        for n in nodes.clone() {
            println!("{n:#?}");
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorPhase, FlavorError};
    use crate::interpreter::{EvalOutcome, EvaluationType, Interpreter};
    use crate::types::{ASTNode, Token, TokenName as TN, Type};

    fn lex_source(source: &str) -> Result<Vec<Token>, FlavorError> {
        let mut lexer = Lexer::new(source);
        lexer.lexe()?;
        Ok(lexer.tokens)
    }

    fn token_names(source: &str) -> Vec<TN> {
        lex_source(source)
            .map(|tokens| {
                tokens
                    .into_iter()
                    .map(|token| token.tok_name)
                    .collect::<Vec<_>>()
            })
            .expect("lexing failed during test setup")
    }

    fn parse_source(source: &str) -> Result<Vec<ASTNode>, FlavorError> {
        let tokens = lex_source(source)?;
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    fn type_check_nodes(nodes: &[ASTNode]) -> Result<(), FlavorError> {
        let mut checker = TypeChecker::new();
        checker.check_program(nodes)
    }

    fn compile_source(source: &str) -> Result<Vec<ASTNode>, FlavorError> {
        let nodes = parse_source(source)?;
        type_check_nodes(&nodes)?;
        Ok(nodes)
    }

    fn evaluate_source(source: &str) -> Result<EvaluationType, FlavorError> {
        let nodes = compile_source(source)?;
        let mut interpreter = Interpreter::new();
        match interpreter.eval_program(&nodes)? {
            EvalOutcome::Value(value) => Ok(value),
            EvalOutcome::Return(value) => Ok(value),
            EvalOutcome::Break => Err(FlavorError::new(
                ErrorPhase::Runtime,
                "Top-level break is not allowed",
                None,
            )),
        }
    }

    #[test]
    fn lexer_recognizes_keywords_and_symbols() {
        let names = token_names(
            "let score = 10; if score >= 5 && score < 20 { print score; } else { score = score - 1; }",
        );
        assert_eq!(
            names,
            vec![
                TN::Let,
                TN::Identifier,
                TN::Assign,
                TN::Number,
                TN::Semicolon,
                TN::If,
                TN::Identifier,
                TN::Ge,
                TN::Number,
                TN::And,
                TN::Identifier,
                TN::Lt,
                TN::Number,
                TN::LBra,
                TN::Print,
                TN::Identifier,
                TN::Semicolon,
                TN::RBra,
                TN::Else,
                TN::LBra,
                TN::Identifier,
                TN::Assign,
                TN::Identifier,
                TN::Minus,
                TN::Number,
                TN::Semicolon,
                TN::RBra,
                TN::Eof,
            ]
        );
    }

    #[test]
    fn lexer_handles_literals_and_delimiters() {
        let names = token_names(
            r#"alias => value; fn f() -> [string] { print "hi", data[0]; return nothing; }"#,
        );
        assert_eq!(
            names,
            vec![
                TN::Alias,
                TN::BoldArrow,
                TN::Identifier,
                TN::Semicolon,
                TN::Fn,
                TN::Identifier,
                TN::LPar,
                TN::RPar,
                TN::SlimArrow,
                TN::LSqu,
                TN::String,
                TN::RSqu,
                TN::LBra,
                TN::Print,
                TN::StringLiteral,
                TN::Comma,
                TN::Identifier,
                TN::LSqu,
                TN::Number,
                TN::RSqu,
                TN::Semicolon,
                TN::Return,
                TN::Nothing,
                TN::Semicolon,
                TN::RBra,
                TN::Eof,
            ]
        );
    }

    #[test]
    fn parser_builds_function_declaration_ast() {
        let source = r#"
fn add(a: int, b: int) -> int {
    let tmp: int = a + b;
    return tmp;
}
"#;
        let nodes = parse_source(source).expect("failed to parse function declaration");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0], ("a".to_string(), Type::Int));
                assert_eq!(parameters[1], ("b".to_string(), Type::Int));
                assert_eq!(*return_type, Type::Int);
                match body.as_ref() {
                    ASTNode::Body {
                        nodes: body_nodes, ..
                    } => {
                        assert_eq!(body_nodes.len(), 2);
                        assert!(matches!(body_nodes[0], ASTNode::LetDeclaration { .. }));
                        assert!(matches!(body_nodes[1], ASTNode::Return { .. }));
                    }
                    other => panic!("expected function body, found {other:?}"),
                }
            }
            other => panic!("expected function declaration, found {other:?}"),
        }
    }

    #[test]
    fn parser_handles_if_else_blocks() {
        let source = r#"
if true {
    print 1;
} else {
    while false {
        break;
    }
}
"#;
        let nodes = parse_source(source).expect("failed to parse conditional");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            ASTNode::If {
                guard,
                then_body,
                else_body,
                ..
            } => {
                assert!(
                    matches!(guard.as_ref(), ASTNode::BoolLiteral { value, .. } if value == "true")
                );
                match then_body.as_ref() {
                    ASTNode::Body {
                        nodes: then_nodes, ..
                    } => {
                        assert_eq!(then_nodes.len(), 1);
                        assert!(matches!(then_nodes[0], ASTNode::Print { .. }));
                    }
                    other => panic!("expected then body, found {other:?}"),
                }
                let else_body = else_body
                    .as_ref()
                    .expect("else branch should have been parsed");
                match else_body.as_ref() {
                    ASTNode::Body {
                        nodes: else_nodes, ..
                    } => {
                        assert_eq!(else_nodes.len(), 1);
                        assert!(matches!(else_nodes[0], ASTNode::While { .. }));
                    }
                    other => panic!("expected else body, found {other:?}"),
                }
            }
            other => panic!("expected if statement, found {other:?}"),
        }
    }

    #[test]
    fn typechecker_accepts_well_typed_program() {
        let source = r#"
fn add(a: int, b: int) -> int {
    return a + b;
}
let answer: int = add(40, 2);
print answer;
"#;
        let nodes = parse_source(source).expect("failed to parse program");
        type_check_nodes(&nodes).expect("program should type check");
    }

    #[test]
    fn typechecker_rejects_assignment_type_mismatch() {
        let nodes =
            parse_source("let x: int = true;").expect("failed to parse assignment mismatch source");
        let err =
            type_check_nodes(&nodes).expect_err("type checker should reject mismatched types");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message.contains("Type mismatch"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_flags_missing_return() {
        let source = r#"
fn bad(n: int) -> int {
    if n > 0 {
        return n;
    }
}
"#;
        let nodes = parse_source(source).expect("failed to parse missing return function");
        let err = type_check_nodes(&nodes).expect_err("type checker should require returns");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message.contains("does not guarantee a return"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_detects_array_element_type_mismatch() {
        let nodes = parse_source("let xs: [int] = [1, true];")
            .expect("failed to parse array literal source");
        let err = type_check_nodes(&nodes).expect_err("type checker should reject mixed arrays");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message
                .contains("Array elements must be of the same type")
                || err.message.contains("Array literal element type mismatch"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_requires_boolean_condition_in_while() {
        let source = r#"
let counter: int = 0;
while counter {
    break;
}
"#;
        let nodes = parse_source(source).expect("failed to parse while loop");
        let err = type_check_nodes(&nodes).expect_err("type checker should reject non-bool guard");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message.contains("While"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_rejects_non_boolean_if_guard() {
        let source = r#"
if 1 {
    print 1;
}
"#;
        let nodes = parse_source(source).expect("failed to parse if statement");
        let err = type_check_nodes(&nodes).expect_err("type checker should reject non-bool guard");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message.contains("If"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_detects_function_argument_mismatch() {
        let source = r#"
fn add(a: int, b: int) -> int {
    return a + b;
}
let value = add(1);
"#;
        let nodes = parse_source(source).expect("failed to parse mismatched call");
        let err =
            type_check_nodes(&nodes).expect_err("type checker should reject arity mismatches");
        assert!(matches!(err.phase, ErrorPhase::TypeChecking));
        assert!(
            err.message.contains("argument count"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn typechecker_accepts_matrix_manipulation() {
        let source = r#"
let matrix: [[int]] = [[1, 2], [3, 4]];
matrix[0][1] = matrix[1][0];
matrix[1][1]++;
let value: int = matrix[0][1] + matrix[1][1];
"#;
        let nodes = parse_source(source).expect("failed to parse matrix manipulation source");
        type_check_nodes(&nodes).expect("matrix program should type check");
    }

    #[test]
    fn interpreter_evaluates_arithmetic_and_assignment() {
        let source = r#"
let x: int = 1;
x = x + 41;
x;
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 42),
            other => panic!("expected integer result, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_supports_closures_and_calls() {
        let source = r#"
let base: int = 10;
let make_adder = <offset: int> -> (int) -> int {
    return <value: int> -> int {
        return value + offset + base;
    };
};
let add_five = make_adder(5);
add_five(3);
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 18),
            other => panic!("expected integer result, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_handles_while_and_break() {
        let source = r#"
let total: int = 0;
let current: int = 0;
while true {
    if current == 5 {
        break;
    }
    total = total + current;
    current++;
}
total;
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 10),
            other => panic!("expected integer result, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_performs_array_indexing_and_assignment() {
        let source = r#"
let values: [int] = [0, 1, 2];
values[1] = 10;
values[1];
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 10),
            other => panic!("expected integer result, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_reports_runtime_errors_for_out_of_bounds_access() {
        let source = r#"
let data: [int] = [0];
data[5];
"#;
        let err = evaluate_source(source).expect_err("runtime error expected");
        assert!(matches!(err.phase, ErrorPhase::Runtime));
        assert!(
            err.message.contains("out of bounds"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn interpreter_executes_recursive_function() {
        let source = r#"
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
factorial(5);
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 120),
            other => panic!("expected integer result, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_manages_matrices_with_mixed_increments() {
        let source = r#"
let matrix: [[int]] = [[1, 2], [3, 4]];
matrix[0][1] = matrix[1][0];
matrix[1][1]++;
++matrix[0][0];
matrix[0][1] + matrix[1][1] + matrix[0][0];
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 10),
            other => panic!("expected integer result after matrix manipulation, found {other:?}"),
        }
    }

    #[test]
    fn interpreter_closure_clones_share_state() {
        let source = r#"
let make_tick = <start: int> -> () -> int {
    let value: int = start;
    return <> -> int {
        ++value;
        return value;
    };
};

let ticker = make_tick(7);
let twin = ticker;
ticker();
twin();
ticker();
"#;
        match evaluate_source(source).expect("program should run") {
            EvaluationType::Int(value) => assert_eq!(value, 10),
            other => {
                panic!("expected integer result showing shared closure state, found {other:?}")
            }
        }
    }
}
