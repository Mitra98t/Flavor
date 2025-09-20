#![allow(unused)]

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub tok_name: TokenName,
    pub lexeme: String,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenName {
    // Built-in Functions
    Print,

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
    Array,

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
    Array(Box<Type>),
    Function {
        param_types: Vec<Type>,
        return_type: Box<Type>,
    },
    // Unknown,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Print(Vec<ASTNode>),
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
    FunctionExpression {
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
    ArrayLiteral(Vec<ASTNode>),
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
        ASTNode::Print(args) => {
            println!("{indent_str}Print:");
            for arg in args.iter() {
                print_node(arg, indent + 1);
            }
        }
        ASTNode::If {
            guard,
            then_body,
            else_body,
        } => {
            println!("{indent_str}If:");
            println!("{indent_str}  Guard:");
            print_node(guard, indent + 2);
            println!("{indent_str}  Then:");
            print_node(then_body, indent + 2);
            if let Some(else_body) = else_body {
                println!("{indent_str}  Else:");
                print_node(else_body, indent + 2);
            }
        }
        ASTNode::While { guard, body } => {
            println!("{indent_str}While:");
            println!("{indent_str}  Guard:");
            print_node(guard, indent + 2);
            println!("{indent_str}  Body:");
            print_node(body, indent + 2);
        }
        ASTNode::Body { nodes } => {
            for n in nodes {
                print_node(n, indent);
            }
        }
        ASTNode::Break => {
            println!("{indent_str}Break");
        }
        ASTNode::Return(expr) => {
            println!("{indent_str}Return:");
            print_node(expr, indent + 2);
        }
        ASTNode::LetDeclaration {
            identifier,
            var_type,
            expr,
        } => {
            println!("{indent_str}LetDeclaration:");
            println!("{indent_str}  Identifier: {identifier}");
            if let Some(t) = var_type {
                println!("{indent_str}  Type: {t:?}");
            } else {
                println!("{indent_str}  Type: None");
            }
            println!("{indent_str}  Expression:");
            print_node(expr, indent + 2);
        }
        ASTNode::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        } => {
            println!("{indent_str}FunctionDeclaration:");
            println!("{indent_str}  Name: {name}");
            println!("{indent_str}  Parameters:");
            for p in parameters {
                println!("{}    Name: {}", indent_str, p.0);
                println!("{}    Type: {:?}", indent_str, p.1);
            }
            println!("{indent_str}  Return Type: {return_type:?}");
            print_node(body, indent + 2);
        }
        ASTNode::FunctionExpression {
            parameters,
            return_type,
            body,
        } => {
            println!("{indent_str}FunctionExpression:");
            println!("{indent_str}  Parameters:");
            for p in parameters {
                println!("{}    Name: {}", indent_str, p.0);
                println!("{}    Type: {:?}", indent_str, p.1);
            }
            println!("{indent_str}  Return Type: {return_type:?}");
            println!("{indent_str}  Body:");
            print_node(body, indent + 2);
        }
        ASTNode::FunctionCall { callee, arguments } => {
            println!("{indent_str}FunctionCall:");
            println!("{indent_str}  Callee:");
            print_node(callee, indent + 2);
            println!("{indent_str}  Arguments:");
            for a in arguments {
                print_node(a, indent + 2);
            }
        }
        ASTNode::UnitLiteral => {}
        ASTNode::NumberLiteral(value) => {
            println!("{indent_str}NumberLiteral: {value}");
        }
        ASTNode::BoolLiteral(value) => {
            println!("{indent_str}BoolLiteral: {value}");
        }
        ASTNode::StringLiteral(value) => {
            println!("{indent_str}StringLiteral: {value}");
        }
        ASTNode::Identifier(name) => {
            println!("{indent_str}Identifier: {name}");
        }
        ASTNode::ArrayLiteral(elements) => {
            println!("{indent_str}ArrayLiteral:");
            for (i, elem) in elements.iter().enumerate() {
                println!("{indent_str}  Element {i}:");
                print_node(elem, indent + 2);
            }
        }
        ASTNode::ArrayAccess { array, index } => {
            println!("{indent_str}ArrayAccess");
            println!("{indent_str}  Array:");
            print_node(array, indent + 2);
            println!("{indent_str}  Index:");
            print_node(index, indent + 2);
        }
        ASTNode::BinaryExpression {
            left,
            operator,
            right,
        } => {
            println!("{indent_str}BinaryExpression: {operator}");
            println!("{indent_str}  Left:");
            print_node(left, indent + 2);
            println!("{indent_str}  Right:");
            print_node(right, indent + 2);
        }
        ASTNode::UnaryExpression {
            operator,
            operand,
            is_postfix,
        } => {
            println!("{indent_str}UnaryExpression: {operator}");
            println!(
                "{}  Is Postfix: {}",
                indent_str,
                if *is_postfix { "true" } else { "false" }
            );
            println!("{indent_str}  operand:");
            print_node(operand, indent + 2);
        }
        ASTNode::ExpressionStatement(expr) => {
            println!("{indent_str}ExpressionStatement:");
            print_node(expr, indent + 1);
        }
    }
}
