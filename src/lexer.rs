use crate::{
    error::{ErrorPhase, FlavorError},
    types::{Span, Token, TokenName as TN},
};
use regex::Regex;

pub struct Lexer {
    pub tokens: Vec<Token>,
    pos: usize,
    source: String,
    line: usize, // Line and column tracking can be added for better error reporting
    column: usize,
}

impl Lexer {
    pub fn new(source_code: &str) -> Self {
        Lexer {
            tokens: vec![],
            pos: 0,
            source: source_code.to_string(),
            line: 1,
            column: 1,
        }
    }
    pub fn lexe(&mut self) -> Result<(), FlavorError> {
        loop {
            let tok = self.next_token()?;

            self.tokens.push(tok.clone());
            if tok.tok_name == TN::Eof {
                break;
            }
        }
        Ok(())
    }

    fn next_token(&mut self) -> Result<Token, FlavorError> {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Ok(Token {
                tok_name: TN::Eof,
                lexeme: "\0".to_string(),
                span: Span::point(self.line, self.column),
            });
        }

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

        let mut token_name = TN::Unknown;
        let mut lexeme = String::new();

        for (pattern, name) in patterns.iter() {
            if let Some(matched) = self.match_start(pattern) {
                token_name = name.clone();
                lexeme = matched.to_string();
                break;
            }
        }

        if token_name == TN::Unknown {
            let span = self.advance_with(&lexeme);
            return Err(FlavorError::with_span(
                ErrorPhase::Lexing,
                format!("Unknown token: {lexeme}"),
                span,
            ));
        };

        let span = self.advance_with(&lexeme);

        Ok(Token {
            tok_name: token_name,
            lexeme,
            span,
        })
    }

    fn advance_with(&mut self, text: &str) -> Span {
        let start_line = self.line;
        let start_column = self.column;

        let mut line = start_line;
        let mut column = start_column;
        let mut end_line = start_line;
        let mut end_column = start_column;

        for ch in text.chars() {
            end_line = line;
            end_column = column;
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        self.pos += text.len();
        self.line = line;
        self.column = column;

        if text.is_empty() {
            end_line = start_line;
            end_column = start_column;
        }

        Span::new(start_line, start_column, end_line, end_column)
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
