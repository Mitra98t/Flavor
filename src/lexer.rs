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

    fn match_or(&self, patterns: &[&str]) -> Option<&str> {
        for p in patterns {
            if let Some(len) = self.match_start(p) {
                return Some(len);
            }
        }
        None
    }

    fn match_start(&self, pattern: &str) -> Option<&str> {
        let re = Regex::new(pattern).unwrap();
        let sub_source = &self.source[self.pos..];
        if let Some(mat) = re.find(sub_source) {
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
        let re = Regex::new(r"^\s$").unwrap();
        while re.is_match(&self.char().to_string()) {
            self.consume_char();
        }
    }

    fn char(&self) -> char {
        self.source.chars().nth(self.pos).unwrap_or('\0')
    }

    fn consume_n_char(&mut self, n: usize) {
        self.pos += n;
    }

    fn consume_char(&mut self) {
        self.pos += 1;
    }
}
