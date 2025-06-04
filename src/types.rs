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
    Return,
    If,
    Else,
    While,
    Break,

    // Types
    Int,
    Float,
    Bool,
    String,
    Nothing,

    // Symbols
    Dot,
    Comma,
    Colon,
    Semicolon,
    SlimArrow,
    BoldArrow,
    Assign,
    Eq,
    NotEq,
    Not,
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
    And,
    Or,

    // Parentheses
    LPar,
    RPar,
    LSqu,
    RSqu,
    LBra,
    RBra,

    // Complex Elements
    Number,
    StringLiteral,
    Identifier,
    True,
    False,

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
    Function {
        param_types: Vec<Type>,
        return_type: Box<Type>,
    },
    // Unknown,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Body {
        nodes: Vec<ASTNode>,
    },
    If {
        guard: Box<ASTNode>,
        then_body: Box<ASTNode>,
        else_body: Option<Box<ASTNode>>,
    },
    While {
        guard: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    LetDeclaration {
        identifier: String,
        var_type: Option<Type>,
        expr: Box<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, Type)>,
        return_type: Type,
        body: Box<ASTNode>,
    },
    Return(Box<ASTNode>),
    Break,
    FunctionCall {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    UnitLiteral,
    NumberLiteral(String),
    StringLiteral(String),
    BoolLiteral(String),
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
        ASTNode::If {
            guard,
            then_body,
            else_body,
        } => {
            println!("{}If:", indent_str);
            println!("{}  Guard:", indent_str);
            print_node(guard, indent + 2);
            println!("{}  Then:", indent_str);
            print_node(then_body, indent + 2);
            if let Some(else_body) = else_body {
                println!("{}  Else:", indent_str);
                print_node(else_body, indent + 2);
            }
        }
        ASTNode::While { guard, body } => {
            println!("{}While:", indent_str);
            println!("{}  Guard:", indent_str);
            print_node(guard, indent + 2);
            println!("{}  Body:", indent_str);
            print_node(body, indent + 2);
        }
        ASTNode::Body { nodes } => {
            for n in nodes {
                print_node(n, indent);
            }
        }
        ASTNode::Break => {
            println!("{}Break", indent_str);
        }
        ASTNode::Return(expr) => {
            println!("{}Return:", indent_str);
            print_node(expr, indent + 2);
        }
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
        ASTNode::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        } => {
            println!("{}FunctionDeclaration:", indent_str);
            println!("{}  Name: {}", indent_str, name);
            println!("{}  Parameters:", indent_str);
            for p in parameters {
                println!("{}    Name: {}", indent_str, p.0);
                println!("{}    Type: {:?}", indent_str, p.1);
            }
            println!("{}  Return Type: {:?}", indent_str, return_type);
            print_node(body, indent + 2);
        }
        ASTNode::FunctionCall { callee, arguments } => {
            println!("{}FunctionCall:", indent_str);
            println!("{}  Callee:", indent_str);
            print_node(callee, indent + 2);
            println!("{}  Arguments:", indent_str);
            for a in arguments {
                print_node(a, indent + 2);
            }
        }
        ASTNode::UnitLiteral => {}
        ASTNode::NumberLiteral(value) => {
            println!("{}NumberLiteral: {}", indent_str, value);
        }
        ASTNode::BoolLiteral(value) => {
            println!("{}BoolLiteral: {}", indent_str, value);
        }
        ASTNode::StringLiteral(value) => {
            println!("{}StringLiteral: {}", indent_str, value);
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
