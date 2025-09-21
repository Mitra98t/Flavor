#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    pub fn point(line: usize, column: usize) -> Self {
        Self::new(line, column, line, column)
    }

    pub fn merge(self, other: Span) -> Self {
        let (start_line, start_column) = if (other.start_line < self.start_line)
            || (other.start_line == self.start_line && other.start_column < self.start_column)
        {
            (other.start_line, other.start_column)
        } else {
            (self.start_line, self.start_column)
        };

        let (end_line, end_column) = if (other.end_line > self.end_line)
            || (other.end_line == self.end_line && other.end_column > self.end_column)
        {
            (other.end_line, other.end_column)
        } else {
            (self.end_line, self.end_column)
        };

        Self::new(start_line, start_column, end_line, end_column)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub tok_name: TokenName,
    pub lexeme: String,
    pub span: Span,
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
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Print {
        expressions: Vec<ASTNode>,
        span: Span,
    },
    Body {
        nodes: Vec<ASTNode>,
        span: Span,
    },
    If {
        guard: Box<ASTNode>,
        then_body: Box<ASTNode>,
        else_body: Option<Box<ASTNode>>,
        span: Span,
    },
    While {
        guard: Box<ASTNode>,
        body: Box<ASTNode>,
        span: Span,
    },
    LetDeclaration {
        identifier: String,
        var_type: Option<Type>,
        expr: Box<ASTNode>,
        span: Span,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, Type)>,
        return_type: Type,
        body: Box<ASTNode>,
        span: Span,
    },
    FunctionExpression {
        parameters: Vec<(String, Type)>,
        return_type: Type,
        body: Box<ASTNode>,
        span: Span,
    },
    Return {
        expr: Box<ASTNode>,
        span: Span,
    },
    Break {
        span: Span,
    },
    FunctionCall {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
        span: Span,
    },
    UnitLiteral {
        span: Span,
    },
    NumberLiteral {
        value: String,
        span: Span,
    },
    StringLiteral {
        value: String,
        span: Span,
    },
    BoolLiteral {
        value: String,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    ArrayLiteral {
        elements: Vec<ASTNode>,
        span: Span,
    },
    ArrayAccess {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
        span: Span,
    },
    BinaryExpression {
        left: Box<ASTNode>,
        operator: String,
        right: Box<ASTNode>,
        span: Span,
    },
    UnaryExpression {
        operator: String,
        operand: Box<ASTNode>,
        is_postfix: bool,
        span: Span,
    },
    ExpressionStatement {
        expr: Box<ASTNode>,
        span: Span,
    },
}

impl ASTNode {
    pub fn span(&self) -> Span {
        match self {
            ASTNode::Print { span, .. }
            | ASTNode::Body { span, .. }
            | ASTNode::If { span, .. }
            | ASTNode::While { span, .. }
            | ASTNode::LetDeclaration { span, .. }
            | ASTNode::FunctionDeclaration { span, .. }
            | ASTNode::FunctionExpression { span, .. }
            | ASTNode::Return { span, .. }
            | ASTNode::Break { span }
            | ASTNode::FunctionCall { span, .. }
            | ASTNode::UnitLiteral { span }
            | ASTNode::NumberLiteral { span, .. }
            | ASTNode::StringLiteral { span, .. }
            | ASTNode::BoolLiteral { span, .. }
            | ASTNode::Identifier { span, .. }
            | ASTNode::ArrayLiteral { span, .. }
            | ASTNode::ArrayAccess { span, .. }
            | ASTNode::BinaryExpression { span, .. }
            | ASTNode::UnaryExpression { span, .. }
            | ASTNode::ExpressionStatement { span, .. } => *span,
        }
    }
}

pub fn print_ast(ast: Vec<ASTNode>) {
    for node in ast.iter() {
        print_node(node, 0);
    }
}

fn print_node(node: &ASTNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    match node {
        ASTNode::Print { expressions, .. } => {
            println!("{indent_str}Print:");
            for arg in expressions.iter() {
                print_node(arg, indent + 1);
            }
        }
        ASTNode::If {
            guard,
            then_body,
            else_body,
            ..
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
        ASTNode::While { guard, body, .. } => {
            println!("{indent_str}While:");
            println!("{indent_str}  Guard:");
            print_node(guard, indent + 2);
            println!("{indent_str}  Body:");
            print_node(body, indent + 2);
        }
        ASTNode::Body { nodes, .. } => {
            for n in nodes {
                print_node(n, indent);
            }
        }
        ASTNode::LetDeclaration {
            identifier,
            var_type,
            expr,
            ..
        } => {
            if let Some(var_type) = var_type {
                println!("{indent_str}Let {identifier}: {var_type:?} =");
            } else {
                println!("{indent_str}Let {identifier} =");
            }
            print_node(expr, indent + 1);
        }
        ASTNode::Break { .. } => {
            println!("{indent_str}Break");
        }
        ASTNode::Return { expr, .. } => {
            println!("{indent_str}Return:");
            print_node(expr, indent + 2);
        }
        ASTNode::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
            ..
        } => {
            println!("{indent_str}FunctionDeclaration: {name}");
            println!("{indent_str}  Parameters:");
            for (param_name, param_ty) in parameters.iter() {
                println!("{indent_str}    {param_name}: {param_ty:?}");
            }
            println!("{indent_str}  Returns: {return_type:?}");
            println!("{indent_str}  Body:");
            print_node(body, indent + 2);
        }
        ASTNode::FunctionExpression {
            parameters,
            return_type,
            body,
            ..
        } => {
            println!("{indent_str}FunctionExpression:");
            println!("{indent_str}  Parameters:");
            for (param_name, param_ty) in parameters.iter() {
                println!("{indent_str}    {param_name}: {param_ty:?}");
            }
            println!("{indent_str}  Returns: {return_type:?}");
            println!("{indent_str}  Body:");
            print_node(body, indent + 2);
        }
        ASTNode::ArrayLiteral { elements, .. } => {
            println!("{indent_str}ArrayLiteral:");
            for elem in elements {
                print_node(elem, indent + 1);
            }
        }
        ASTNode::ArrayAccess { array, index, .. } => {
            println!("{indent_str}ArrayAccess:");
            println!("{indent_str}  Array:");
            print_node(array, indent + 2);
            println!("{indent_str}  Index:");
            print_node(index, indent + 2);
        }
        ASTNode::BinaryExpression {
            left,
            operator,
            right,
            ..
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
            ..
        } => {
            println!("{indent_str}UnaryExpression: {operator} postfix={is_postfix}");
            print_node(operand, indent + 1);
        }
        ASTNode::FunctionCall {
            callee, arguments, ..
        } => {
            println!("{indent_str}FunctionCall:");
            println!("{indent_str}  Callee:");
            print_node(callee, indent + 2);
            println!("{indent_str}  Arguments:");
            for arg in arguments {
                print_node(arg, indent + 2);
            }
        }
        ASTNode::UnitLiteral { .. } => {
            println!("{indent_str}<unit>");
        }
        ASTNode::NumberLiteral { value, .. } => {
            println!("{indent_str}Number({value})");
        }
        ASTNode::StringLiteral { value, .. } => {
            println!("{indent_str}String({value})");
        }
        ASTNode::BoolLiteral { value, .. } => {
            println!("{indent_str}Bool({value})");
        }
        ASTNode::Identifier { name, .. } => {
            println!("{indent_str}Identifier({name})");
        }
        ASTNode::ExpressionStatement { expr, .. } => {
            println!("{indent_str}ExpressionStatement:");
            print_node(expr, indent + 1);
        }
    }
}
