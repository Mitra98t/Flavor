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

#[derive(Debug, Clone)]
pub enum ASTNode {
    LetDeclaration {
        identifier: String,
        var_type: Option<String>,
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
                println!("{}  Type: {}", indent_str, t);
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
