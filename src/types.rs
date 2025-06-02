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
