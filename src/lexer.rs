use crate::types::{Token, TokenName as TN};
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
            if tok.tok_name == TN::Eof {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Token {
                tok_name: TN::Eof,
                lexeme: "\0".to_string(),
            };
        }

        let mut tok = Token {
            tok_name: TN::Unknown,
            lexeme: "".to_string(),
        };

        let mut length_of_tok: usize = 0;

        let patterns = [
            (r"print\b", TN::Print),
            (r"let\b", TN::Let),
            (r"fn\b", TN::Fn),
            (r"alias\b", TN::Alias),
            (r"int\b", TN::Int),
            (r"float\b", TN::Float),
            (r"string\b", TN::String),
            (r"bool\b", TN::Bool),
            (r"array\b", TN::Array),
            (r"return\b", TN::Return),
            (r"break\b", TN::Break),
            (r"if\b", TN::If),
            (r"else\b", TN::Else),
            (r"while\b", TN::While),
            (r"nothing\b", TN::Nothing),
            (r"true\b", TN::True),
            (r"false\b", TN::False),
            (r"\.", TN::Dot),
            (r",", TN::Comma),
            (r":", TN::Colon),
            (r";", TN::Semicolon),
            (r"->", TN::SlimArrow),
            (r"=>", TN::BoldArrow),
            (r"==", TN::Eq),
            (r"\!=", TN::NotEq),
            (r"\=", TN::Assign),
            (r"\!", TN::Not),
            (r">=", TN::Ge),
            (r"<=", TN::Le),
            (r">", TN::Gt),
            (r"<", TN::Lt),
            (r"\+\+", TN::PlusPlus),
            (r"--", TN::MinusMinus),
            (r"\+", TN::Plus),
            (r"-", TN::Minus),
            (r"\*", TN::Times),
            (r"/", TN::Div),
            (r"%", TN::Percent),
            (r"&&", TN::And),
            (r"\|\|", TN::Or),
            (r"\(", TN::LPar),
            (r"\)", TN::RPar),
            (r"\[", TN::LSqu),
            (r"\]", TN::RSqu),
            (r"\{", TN::LBra),
            (r"\}", TN::RBra),
            (r"[0-9]+", TN::Number),
            (r#""(.*?)""#, TN::StringLiteral),
            (r"[a-zA-Z_][a-zA-Z0-9_]*", TN::Identifier),
            (r"[\s\S]*", TN::Unknown),
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
