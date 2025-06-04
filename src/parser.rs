use crate::types::{ASTNode, Token, TokenName, Type};

type ParseProduction = Result<ASTNode, String>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current_tok(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn consume_tok(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    /// Checks the current token with the expected token provided.
    /// The method consumes the tokens if it is correct.
    /// Returns a result with the token if success or the error string if failure.
    /// * `expected`: the expected token name
    fn expect_tok(&mut self, expected: TokenName) -> Result<Token, String> {
        let tok = self.current_tok();

        if tok.tok_name == expected {
            let tok = tok.clone();
            self.consume_tok();
            Ok(tok)
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} ('{}')",
                expected, tok.tok_name, tok.lexeme
            ))
        }
    }

    /// produce the precedence for the operators
    ///
    /// * `token`: token to evaluate the precedence of
    fn get_precedence(token: &Token) -> Option<u8> {
        match token.tok_name {
            TokenName::Times | TokenName::Div | TokenName::Percent => Some(150),
            TokenName::Plus | TokenName::Minus => Some(120),
            TokenName::Gt | TokenName::Lt | TokenName::Ge | TokenName::Le => Some(100),
            TokenName::Eq | TokenName::NotEq => Some(80),
            TokenName::And => Some(50),
            TokenName::Or => Some(40),
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut nodes = Vec::new();
        while self.current_tok().tok_name != TokenName::Eof {
            nodes.push(self.parse_statement()?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self) -> ParseProduction {
        match self.current_tok().tok_name {
            TokenName::Let => self.parse_let_statement(),
            TokenName::Fn => self.parse_function_declaration(),
            // TODO: aliasing, ecc...
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_function_declaration(&mut self) -> ParseProduction {
        self.expect_tok(TokenName::Fn)?;
        let fn_name = self.expect_tok(TokenName::Identifier)?;

        let parameters = self.parse_fn_parameters()?;

        self.expect_tok(TokenName::SlimArrow)?;

        let return_ty = self.parse_type()?;

        self.expect_tok(TokenName::LBra)?;
        let mut body_nodes = Vec::new();
        while self.current_tok().tok_name != TokenName::Rbra {
            body_nodes.push(self.parse_statement()?);
        }
        self.expect_tok(TokenName::Rbra)?;

        Ok(ASTNode::FunctionDeclaration {
            name: fn_name.lexeme,
            parameters,
            return_type: return_ty,
            body: body_nodes,
        })
    }

    fn parse_fn_parameters(&mut self) -> Result<Vec<(String, Type)>, String> {
        self.expect_tok(TokenName::LPar)?;
        let mut params: Vec<(String, Type)> = vec![];
        while self.current_tok().tok_name != TokenName::LPar {
            let param_name = self.expect_tok(TokenName::Identifier)?;
            self.expect_tok(TokenName::Colon)?;
            let param_ty = self.parse_type()?;
            params.push((param_name.lexeme, param_ty));
        }

        Ok(params)
    }

    fn parse_let_statement(&mut self) -> ParseProduction {
        self.expect_tok(TokenName::Let)?;
        let id_tok = self.expect_tok(TokenName::Identifier)?;

        // Optional Type definition
        let var_type: Option<Type> = if self.current_tok().tok_name == TokenName::Colon {
            // The type is being defined (there is the colon `:`)
            self.consume_tok();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Necessary initialization??
        self.expect_tok(TokenName::Assign)?;
        let expr = self.parse_expression()?;
        self.expect_tok(TokenName::Semicolon)?;

        // Return of the AST
        Ok(ASTNode::LetDeclaration {
            identifier: id_tok.lexeme,
            var_type,
            expr: Box::new(expr),
        })
    }

    fn parse_expression_statement(&mut self) -> ParseProduction {
        let expr = self.parse_expression()?;
        self.expect_tok(TokenName::Semicolon)?;
        Ok(ASTNode::ExpressionStatement(Box::new(expr)))
    }

    fn parse_expression(&mut self) -> ParseProduction {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_prec: u8) -> ParseProduction {
        let mut left = self.parse_postfix_expression()?;

        while let Some(prec) = Self::get_precedence(self.current_tok()) {
            if prec < min_prec {
                break;
            }

            let op_tok = self.current_tok().clone();
            self.consume_tok();

            let right = self.parse_binary_expression(prec + 1)?;
            left = ASTNode::BinaryExpression {
                left: Box::new(left),
                operator: op_tok.lexeme,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_postfix_expression(&mut self) -> ParseProduction {
        // Parse base expression
        let mut expr = self.parse_unary_expression()?;

        // Loop to handle chaining postfix
        loop {
            match self.current_tok().tok_name {
                TokenName::PlusPlus | TokenName::MinusMinus => {
                    let op_tok = self.current_tok().clone();
                    self.consume_tok();
                    expr = ASTNode::UnaryExpression {
                        operator: op_tok.lexeme,
                        operand: Box::new(expr),
                        is_postfix: true,
                    }
                }
                TokenName::LSqu => {
                    self.consume_tok(); // consume '['
                    let index_expr = self.parse_expression()?;
                    self.expect_tok(TokenName::RSqu)?;
                    expr = ASTNode::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index_expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_unary_expression(&mut self) -> ParseProduction {
        let tok = self.current_tok().clone();
        match tok.tok_name {
            TokenName::MinusMinus | TokenName::PlusPlus | TokenName::Minus | TokenName::Not => {
                // Consume unary operator
                self.consume_tok();

                // dparse operand recursively as unary expression to support chaining
                let operand = self.parse_unary_expression()?;
                Ok(ASTNode::UnaryExpression {
                    operator: tok.lexeme,
                    operand: Box::new(operand),
                    is_postfix: false,
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> ParseProduction {
        let tok = self.current_tok().clone();
        match tok.tok_name {
            TokenName::Number => {
                self.consume_tok();
                Ok(ASTNode::NumberLiteral(tok.lexeme))
            }
            TokenName::True | TokenName::False => {
                self.consume_tok();
                Ok(ASTNode::BoolLiteral(tok.lexeme))
            }
            TokenName::StringLiteral => {
                self.consume_tok();
                Ok(ASTNode::StringLiteral(tok.lexeme))
            }
            TokenName::Identifier => {
                self.consume_tok();
                Ok(ASTNode::Identifier(tok.lexeme))
            }
            TokenName::LPar => {
                self.consume_tok();
                let expr = self.parse_expression()?;
                self.expect_tok(TokenName::RPar)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", tok)),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current_tok().tok_name {
            TokenName::Int => {
                self.consume_tok();
                Ok(Type::Int)
            }
            TokenName::Float => {
                self.consume_tok();
                Ok(Type::Float)
            }
            TokenName::Bool => {
                self.consume_tok();
                Ok(Type::Bool)
            }
            TokenName::String => {
                self.consume_tok();
                Ok(Type::String)
            }
            TokenName::Identifier => {
                let id = self.current_tok().lexeme.clone();
                self.consume_tok();
                Ok(Type::Custom(id))
            }
            _ => Err("Expected a type".to_string()),
        }
    }
}
