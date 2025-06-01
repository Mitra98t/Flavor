#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub tok_name: TokenName,
    pub lexeme: String,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenName {
    Let,
    Fn,
    Alias,

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

    LPar,
    RPar,
    LSqu,
    RSqu,
    LBra,
    Rbra,

    Number,
    Identifier,

    Unknown,
    Eof,
}
