use std::fmt::Display;

#[derive(Clone, PartialEq, Debug, Copy)]
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
        Self {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column,
        }
    }

    pub fn merge(&self, other: &Span) -> Self {
        Self {
            start_line: self.start_line.min(other.start_line),
            start_column: if self.start_line < other.start_line {
                self.start_column
            } else if self.start_line > other.start_line {
                other.start_column
            } else {
                self.start_column.min(other.start_column)
            },
            end_line: self.end_line.max(other.end_line),
            end_column: if self.end_line > other.end_line {
                self.end_column
            } else if self.end_line < other.end_line {
                other.end_column
            } else {
                self.end_column.max(other.end_column)
            },
        }
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
    FloatLiteral,
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
    FloatLiteral {
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
    pub fn span(&self) -> &Span {
        match self {
            ASTNode::Print { span, .. }
            | ASTNode::Body { span, .. }
            | ASTNode::If { span, .. }
            | ASTNode::While { span, .. }
            | ASTNode::LetDeclaration { span, .. }
            | ASTNode::FunctionDeclaration { span, .. }
            | ASTNode::FunctionExpression { span, .. }
            | ASTNode::Return { span, .. }
            | ASTNode::Break { span, .. }
            | ASTNode::FunctionCall { span, .. }
            | ASTNode::UnitLiteral { span, .. }
            | ASTNode::NumberLiteral { span, .. }
            | ASTNode::FloatLiteral { span, .. }
            | ASTNode::StringLiteral { span, .. }
            | ASTNode::BoolLiteral { span, .. }
            | ASTNode::Identifier { span, .. }
            | ASTNode::ArrayLiteral { span, .. }
            | ASTNode::ArrayAccess { span, .. }
            | ASTNode::BinaryExpression { span, .. }
            | ASTNode::UnaryExpression { span, .. }
            | ASTNode::ExpressionStatement { span, .. } => span,
        }
    }
}

impl Display for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Print { expressions, .. } => {
                let exprs: Vec<String> = expressions.iter().map(|e| format!("{e}")).collect();
                write!(f, "print {}", exprs.join(", "))
            }
            ASTNode::Body { nodes, .. } => {
                let nodes_str: Vec<String> = nodes.iter().map(|n| format!("{n}")).collect();
                write!(f, "{{\n{}\n}}", nodes_str.join("\n"))
            }
            ASTNode::If {
                guard,
                then_body,
                else_body,
                ..
            } => {
                if let Some(else_body) = else_body {
                    write!(f, "if ({guard}) {then_body}\nelse {else_body}")
                } else {
                    write!(f, "if ({guard}) {then_body}")
                }
            }
            ASTNode::While { guard, body, .. } => {
                write!(f, "while ({guard}) {body}")
            }
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
                ..
            } => {
                if let Some(var_type) = var_type {
                    write!(f, "let {identifier}: {var_type:?} = {expr};")
                } else {
                    write!(f, "let {identifier} = {expr};")
                }
            }
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                let params: Vec<String> = parameters
                    .iter()
                    .map(|(n, t)| format!("{n}: {t:?}"))
                    .collect();
                write!(
                    f,
                    "fn {}({}) -> {:?} {}",
                    name,
                    params.join(", "),
                    return_type,
                    body
                )
            }
            ASTNode::FunctionExpression {
                parameters,
                return_type,
                body,
                ..
            } => {
                let params: Vec<String> = parameters
                    .iter()
                    .map(|(n, t)| format!("{n}: {t:?}"))
                    .collect();
                write!(f, "fn({}) -> {:?} {}", params.join(", "), return_type, body)
            }
            ASTNode::Return { expr, .. } => {
                write!(f, "return {expr};")
            }
            ASTNode::Break { .. } => {
                write!(f, "break;")
            }
            ASTNode::FunctionCall {
                callee, arguments, ..
            } => {
                let args: Vec<String> = arguments.iter().map(|a| format!("{a}")).collect();
                write!(f, "{}({})", callee, args.join(", "))
            }
            ASTNode::UnitLiteral { .. } => {
                write!(f, "()")
            }
            ASTNode::NumberLiteral { value, .. } => {
                write!(f, "{value}")
            }
            ASTNode::FloatLiteral { value, .. } => {
                write!(f, "{value}")
            }
            ASTNode::StringLiteral { value, .. } => {
                write!(f, "{value}")
            }
            ASTNode::BoolLiteral { value, .. } => {
                write!(f, "{value}")
            }
            ASTNode::Identifier { name, .. } => {
                write!(f, "{name}")
            }
            ASTNode::ArrayLiteral { elements, .. } => {
                let elems: Vec<String> = elements.iter().map(|e| format!("{e}")).collect();
                write!(f, "[{}]", elems.join(", "))
            }
            ASTNode::ArrayAccess { array, index, .. } => {
                write!(f, "{array}[{index}]")
            }
            ASTNode::BinaryExpression {
                left,
                operator,
                right,
                ..
            } => {
                write!(f, "({left} {operator} {right})")
            }
            ASTNode::UnaryExpression {
                operator,
                operand,
                is_postfix,
                ..
            } => {
                if *is_postfix {
                    write!(f, "({operand}{operator})")
                } else {
                    write!(f, "({operator}{operand})")
                }
            }
            ASTNode::ExpressionStatement { expr, .. } => {
                write!(f, "{expr};")
            }
        }
    }
}
