```rust
// main.rs
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
```

```rust
// types.rs

#![allow(unused)]

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub tok_name: TokenName,
    pub lexeme: String,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenName {
    // Keywords
    Let,
    Fn,
    Alias,

    // Types
    Int,
    Float,
    Bool,
    String,
    Nothing,

    // Symbols
    Colon,
    Semicolon,
    Assign,
    Eq,
    NotEq,
    Not,
    SlimArrow,
    BoldArrow,
    Gt,
    Lt,
    Ge,
    Le,
    PlusPlus,
    MinusMinus,
    Plus,
    Minus,
    Times,
    Div,
    Percent,

    // Parentheses
    LPar,
    RPar,
    LSqu,
    RSqu,
    LBra,
    Rbra,

    // Complex Elements
    Number,
    Identifier,

    // Utils
    Unknown,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Float,
    String,
    Unit,
    Custom(String),
    // Array(Box<Type>),
    // Function {
    //     param_types: Vec<Type>,
    //     return_type: Box<Type>,
    // },
    // Unknown,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    LetDeclaration {
        identifier: String,
        var_type: Option<Type>,
        expr: Box<ASTNode>,
    },

    NumberLiteral(String),
    Identifier(String),
    ArrayAccess {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    BinaryExpression {
        left: Box<ASTNode>,
        operator: String,
        right: Box<ASTNode>,
    },
    UnaryExpression {
        operator: String,
        operand: Box<ASTNode>,
        is_postfix: bool,
    },
    ExpressionStatement(Box<ASTNode>),
}

pub fn print_ast(ast: Vec<ASTNode>) {
    for node in ast.iter() {
        print_node(node, 0);
    }
}

fn print_node(node: &ASTNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    match node {
        ASTNode::LetDeclaration {
            identifier,
            var_type,
            expr,
        } => {
            println!("{}LetDeclaration:", indent_str);
            println!("{}  Identifier: {}", indent_str, identifier);
            if let Some(t) = var_type {
                println!("{}  Type: {:?}", indent_str, t);
            } else {
                println!("{}  Type: None", indent_str);
            }
            println!("{}  Expression:", indent_str);
            print_node(expr, indent + 2);
        }
        ASTNode::NumberLiteral(value) => {
            println!("{}NumberLiteral: {}", indent_str, value);
        }
        ASTNode::Identifier(name) => {
            println!("{}Identifier: {}", indent_str, name);
        }
        ASTNode::ArrayAccess { array, index } => {
            println!("{}ArrayAccess", indent_str);
            println!("{}  Array:", indent_str);
            print_node(array, indent + 2);
            println!("{}  Index:", indent_str);
            print_node(index, indent + 2);
        }
        ASTNode::BinaryExpression {
            left,
            operator,
            right,
        } => {
            println!("{}BinaryExpression: {}", indent_str, operator);
            println!("{}  Left:", indent_str);
            print_node(left, indent + 2);
            println!("{}  Right:", indent_str);
            print_node(right, indent + 2);
        }
        ASTNode::UnaryExpression {
            operator,
            operand,
            is_postfix,
        } => {
            println!("{}UnaryExpression: {}", indent_str, operator);
            println!(
                "{}  Is Postfix: {}",
                indent_str,
                if *is_postfix { "true" } else { "false" }
            );
            println!("{}  operand:", indent_str);
            print_node(operand, indent + 2);
        }
        ASTNode::ExpressionStatement(expr) => {
            println!("{}ExpressionStatement:", indent_str);
            print_node(expr, indent + 1);
        }
    }
}
```

```rust
// lexer.rs

use crate::types::{Token, TokenName};
use regex::Regex;

pub struct Lexer {
    pub tokens: Vec<Token>,
    pos: usize,
    source: String,
}

impl Lexer {
    pub fn new(source_code: &str) -> Self {
        Lexer {
            tokens: vec![],
            pos: 0,
            source: source_code.to_string(),
        }
    }
    pub fn lexe(&mut self) {
        loop {
            let tok = self.next_token();

            self.tokens.push(tok.clone());
            if tok.tok_name == TokenName::Eof {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Token {
                tok_name: TokenName::Eof,
                lexeme: "\0".to_string(),
            };
        }

        let mut tok = Token {
            tok_name: TokenName::Unknown,
            lexeme: "".to_string(),
        };

        let mut length_of_tok: usize = 0;

        let patterns = [
            (r"let", TokenName::Let),
            (r"fn", TokenName::Fn),
            (r"alias", TokenName::Alias),
            (r"int", TokenName::Int),
            (r"float", TokenName::Float),
            (r"string", TokenName::String),
            (r"bool", TokenName::Bool),
            (r"noting", TokenName::Nothing),
            (r":", TokenName::Colon),
            (r";", TokenName::Semicolon),
            (r"->", TokenName::SlimArrow),
            (r"=>", TokenName::BoldArrow),
            (r"==", TokenName::Eq),
            (r"\!=", TokenName::NotEq),
            (r"\=", TokenName::Assign),
            (r"\!", TokenName::Not),
            (r">=", TokenName::Ge),
            (r"<=", TokenName::Le),
            (r">", TokenName::Gt),
            (r"<", TokenName::Lt),
            (r"\+\+", TokenName::PlusPlus),
            (r"--", TokenName::MinusMinus),
            (r"\+", TokenName::Plus),
            (r"-", TokenName::Minus),
            (r"\*", TokenName::Times),
            (r"/", TokenName::Div),
            (r"%", TokenName::Percent),
            (r"\(", TokenName::LPar),
            (r"\)", TokenName::RPar),
            (r"\[", TokenName::LSqu),
            (r"\]", TokenName::RSqu),
            (r"\{", TokenName::LBra),
            (r"\}", TokenName::Rbra),
            (r"[0-9]+", TokenName::Number),
            (r"[a-zA-Z_][a-zA-Z0-9_]*", TokenName::Identifier),
            (r"[\s\S]*", TokenName::Unknown),
        ];

        for (pattern, token_name) in patterns.iter() {
            if let Some(lexeme) = self.match_start(pattern) {
                tok.tok_name = token_name.clone();
                tok.lexeme = lexeme.to_string();
                length_of_tok = lexeme.len();
                break;
            }
        }

        self.consume_n_char(length_of_tok);

        tok
    }

    fn match_start(&self, pattern: &str) -> Option<&str> {
        let re = Regex::new(pattern).unwrap();
        if let Some(mat) = re.find(self.remaining_source()) {
            if mat.start() == 0 {
                Some(mat.as_str())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        let re = Regex::new(r"^\s+").unwrap();
        while let Some(m) = re.find(self.remaining_source()) {
            if m.start() == 0 {
                self.consume_n_char(m.end());
            } else {
                break;
            }
        }
    }

    fn remaining_source(&self) -> &str {
        &self.source[self.pos..]
    }

    fn consume_n_char(&mut self, n: usize) {
        self.pos += n;
    }
}

```

```rust
// parser.rs

use crate::types::{ASTNode, Token, TokenName, Type};

type ParseProduction = Result<ASTNode, String>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current_tok(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn consume_tok(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    /// Checks the current token with the expected token provided.
    /// The method consumes the tokens if it is correct.
    /// Returns a result with the token if success or the error string if failure.
    /// * `expected`: the expected token name
    fn expect_tok(&mut self, expected: TokenName) -> Result<Token, String> {
        let tok = self.current_tok();

        if tok.tok_name == expected {
            let tok = tok.clone();
            self.consume_tok();
            Ok(tok)
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} ('{}')",
                expected, tok.tok_name, tok.lexeme
            ))
        }
    }

    /// produce the precedence for the operators
    ///
    /// * `token`: token to evaluate the precedence of
    fn get_precedence(token: &Token) -> Option<u8> {
        match token.tok_name {
            TokenName::Plus | TokenName::Minus => Some(10),
            TokenName::Times | TokenName::Div | TokenName::Percent => Some(20),
            TokenName::Eq | TokenName::NotEq => Some(5),
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut nodes = Vec::new();
        while self.current_tok().tok_name != TokenName::Eof {
            nodes.push(self.parse_statement()?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self) -> ParseProduction {
        match self.current_tok().tok_name {
            TokenName::Let => self.parse_let_statement(),
            // TODO: aliasing, fn declaration ecc...
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> ParseProduction {
        self.expect_tok(TokenName::Let)?;
        let id_tok = self.expect_tok(TokenName::Identifier)?;

        // Optional Type definition
        let var_type: Option<Type> = if self.current_tok().tok_name == TokenName::Colon {
            // The type is being defined (there is the colon `:`)
            self.consume_tok();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Necessary initialization??
        self.expect_tok(TokenName::Assign)?;
        let expr = self.parse_expression()?;
        self.expect_tok(TokenName::Semicolon)?;

        // Return of the AST
        Ok(ASTNode::LetDeclaration {
            identifier: id_tok.lexeme,
            var_type,
            expr: Box::new(expr),
        })
    }

    fn parse_expression_statement(&mut self) -> ParseProduction {
        let expr = self.parse_expression()?;
        self.expect_tok(TokenName::Semicolon)?;
        Ok(ASTNode::ExpressionStatement(Box::new(expr)))
    }

    fn parse_expression(&mut self) -> ParseProduction {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_prec: u8) -> ParseProduction {
        let mut left = self.parse_postfix_expression()?;

        while let Some(prec) = Self::get_precedence(self.current_tok()) {
            if prec < min_prec {
                break;
            }

            let op_tok = self.current_tok().clone();
            self.consume_tok();

            let right = self.parse_binary_expression(prec + 1)?;
            left = ASTNode::BinaryExpression {
                left: Box::new(left),
                operator: op_tok.lexeme,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_postfix_expression(&mut self) -> ParseProduction {
        // Parse base expression
        let mut expr = self.parse_unary_expression()?;

        // Loop to handle chaining postfix
        loop {
            match self.current_tok().tok_name {
                TokenName::PlusPlus | TokenName::MinusMinus => {
                    let op_tok = self.current_tok().clone();
                    self.consume_tok();
                    expr = ASTNode::UnaryExpression {
                        operator: op_tok.lexeme,
                        operand: Box::new(expr),
                        is_postfix: true,
                    }
                }
                TokenName::LSqu => {
                    self.consume_tok(); // consume '['
                    let index_expr = self.parse_expression()?;
                    self.expect_tok(TokenName::RSqu)?;
                    expr = ASTNode::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index_expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_unary_expression(&mut self) -> ParseProduction {
        let tok = self.current_tok().clone();
        match tok.tok_name {
            TokenName::MinusMinus | TokenName::PlusPlus | TokenName::Minus | TokenName::Not => {
                // Consume unary operator
                self.consume_tok();

                // dparse operand recursively as unary expression to support chaining
                let operand = self.parse_unary_expression()?;
                Ok(ASTNode::UnaryExpression {
                    operator: tok.lexeme,
                    operand: Box::new(operand),
                    is_postfix: false,
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> ParseProduction {
        let tok = self.current_tok().clone();
        match tok.tok_name {
            TokenName::Number => {
                self.consume_tok();
                Ok(ASTNode::NumberLiteral(tok.lexeme))
            }
            TokenName::Identifier => {
                self.consume_tok();
                Ok(ASTNode::Identifier(tok.lexeme))
            }
            TokenName::LPar => {
                self.consume_tok();
                let expr = self.parse_expression()?;
                self.expect_tok(TokenName::RPar)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", tok)),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current_tok().tok_name {
            TokenName::Int => {
                self.consume_tok();
                Ok(Type::Int)
            }
            TokenName::Float => {
                self.consume_tok();
                Ok(Type::Float)
            }
            TokenName::Bool => {
                self.consume_tok();
                Ok(Type::Bool)
            }
            TokenName::String => {
                self.consume_tok();
                Ok(Type::String)
            }
            TokenName::Identifier => {
                let id = self.current_tok().lexeme.clone();
                self.consume_tok();
                Ok(Type::Custom(id))
            }
            _ => Err("Expected a type".to_string()),
        }
    }
}
```

```rust
// typechecker.rs

use std::collections::HashMap;

use crate::types::{ASTNode, Type};

pub struct TypeChecker {
    variables: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.variables.insert(name, ty);
    }

    pub fn get(&self, name: String) -> Option<&Type> {
        self.variables.get(&name)
    }

    pub fn check_program(&mut self, nodes: &[ASTNode]) -> Result<(), String> {
        for n in nodes {
            self.check(n)?;
        }
        Ok(())
    }

    fn check(&mut self, node: &ASTNode) -> Result<Type, String> {
        match node {
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                let expr_ty = self.check(expr)?;
                if let Some(declared_ty) = var_type {
                    if *declared_ty != expr_ty {
                        return Err(format!(
                            "Type mismatch in Let Declaration: variable '{}' declared as {:?} but expression has type {:?}",
                            identifier, var_type, expr_ty
                        ));
                    }
                    self.insert(identifier.clone(), declared_ty.clone());
                    Ok(declared_ty.clone())
                } else {
                    self.insert(identifier.clone(), expr_ty.clone());
                    Ok(expr_ty)
                }
            }
            // FIX: how to handle floats?
            ASTNode::NumberLiteral(_) => Ok(Type::Int),
            ASTNode::Identifier(name) => {
                if let Some(t) = self.get(name.to_string()) {
                    Ok(t.clone())
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            ASTNode::ExpressionStatement(expr) => {
                let _ = self.check(expr);
                Ok(Type::Unit)
            }
            ASTNode::BinaryExpression {
                left,
                operator,
                right,
            } => {
                let left_ty = self.check(left)?;
                let right_ty = self.check(right)?;

                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!(
                                "Operator '{}' requires Integer operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "==" | "!=" => {
                        if left_ty == right_ty {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Cannot compare different types. Found left: {:?}, right: {:?}",
                                left_ty, right_ty
                            ))
                        }
                    }
                    _ => Err(format!("Unknown operator '{}'", operator)),
                }
            }
            _ => Err(format!("Type checking not implemented for node {:?}", node)),
        }
    }
}

```
