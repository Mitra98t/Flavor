use crate::types::{ASTNode, Token, TokenName as TN, Type};

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
    fn expect_tok(&mut self, expected: TN) -> Result<Token, String> {
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
            TN::Times | TN::Div | TN::Percent => Some(150),
            TN::Plus | TN::Minus => Some(120),
            TN::Gt | TN::Lt | TN::Ge | TN::Le => Some(100),
            TN::Eq | TN::NotEq => Some(80),
            TN::And => Some(50),
            TN::Or => Some(40),
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut nodes = Vec::new();
        while self.current_tok().tok_name != TN::Eof {
            nodes.push(self.parse_statement()?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self) -> ParseProduction {
        match self.current_tok().tok_name {
            TN::Let => self.parse_let_statement(),
            TN::Fn => self.parse_function_declaration(),
            TN::If => self.parse_if(),
            TN::While => self.parse_while(),
            TN::Return => self.parse_return(),
            TN::Break => self.parse_break(),
            TN::LBra => self.parse_body(),
            // TODO: aliasing, ecc...
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_while(&mut self) -> ParseProduction {
        self.expect_tok(TN::While)?;
        let guard = self.parse_expression()?;
        let body = self.parse_body()?;

        Ok(ASTNode::While {
            guard: Box::new(guard),
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> ParseProduction {
        self.expect_tok(TN::If)?;
        let guard = self.parse_expression()?;
        let then_body = self.parse_body()?;
        let mut else_body = None;
        if self.current_tok().tok_name == TN::Else {
            else_body = Some(Box::new(self.parse_body()?));
        }
        // HACK: IF WITH Semicolon??
        // self.expect_tok(TN::Semicolon)?;
        Ok(ASTNode::If {
            guard: Box::new(guard),
            then_body: Box::new(then_body),
            else_body,
        })
    }
    fn parse_break(&mut self) -> ParseProduction {
        self.expect_tok(TN::Break)?;
        self.expect_tok(TN::Semicolon)?;
        Ok(ASTNode::Break)
    }

    fn parse_return(&mut self) -> ParseProduction {
        self.expect_tok(TN::Return)?;
        let mut expr = Box::new(ASTNode::UnitLiteral);
        if self.current_tok().tok_name != TN::Semicolon {
            expr = Box::new(self.parse_expression()?);
        }
        self.expect_tok(TN::Semicolon)?;
        Ok(ASTNode::Return(expr))
    }

    fn parse_function_declaration(&mut self) -> ParseProduction {
        self.expect_tok(TN::Fn)?;
        let fn_name = self.expect_tok(TN::Identifier)?;

        let parameters = self.parse_fn_parameters()?;

        self.expect_tok(TN::SlimArrow)?;

        let return_ty = self.parse_type()?;

        let body = self.parse_body()?;

        Ok(ASTNode::FunctionDeclaration {
            name: fn_name.lexeme,
            parameters,
            return_type: return_ty,
            body: Box::new(body),
        })
    }

    fn parse_body(&mut self) -> Result<ASTNode, String> {
        assert_eq!(TN::LBra, self.current_tok().tok_name);
        self.expect_tok(TN::LBra)?;

        let mut statements: Vec<ASTNode> = vec![];
        while self.current_tok().tok_name != TN::RBra {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.expect_tok(TN::RBra)?;
        Ok(ASTNode::Body { nodes: statements })
    }

    fn parse_fn_parameters(&mut self) -> Result<Vec<(String, Type)>, String> {
        self.expect_tok(TN::LPar)?;
        let mut params: Vec<(String, Type)> = vec![];
        loop {
            if self.current_tok().tok_name == TN::RPar {
                break;
            }
            let param_name = self.expect_tok(TN::Identifier)?;
            self.expect_tok(TN::Colon)?;
            let param_ty = self.parse_type()?;
            params.push((param_name.lexeme, param_ty));
            if self.current_tok().tok_name == TN::Comma {
                self.consume_tok();
            }
        }

        self.expect_tok(TN::RPar)?;

        Ok(params)
    }

    fn parse_let_statement(&mut self) -> ParseProduction {
        self.expect_tok(TN::Let)?;
        let id_tok = self.expect_tok(TN::Identifier)?;

        // Optional Type definition
        let var_type: Option<Type> = if self.current_tok().tok_name == TN::Colon {
            // The type is being defined (there is the colon `:`)
            self.consume_tok();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Necessary initialization??
        self.expect_tok(TN::Assign)?;
        let expr = self.parse_expression()?;
        self.expect_tok(TN::Semicolon)?;

        // Return of the AST
        Ok(ASTNode::LetDeclaration {
            identifier: id_tok.lexeme,
            var_type,
            expr: Box::new(expr),
        })
    }

    fn parse_expression_statement(&mut self) -> ParseProduction {
        let expr = self.parse_expression()?;
        self.expect_tok(TN::Semicolon)?;
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
                TN::PlusPlus | TN::MinusMinus => {
                    let op_tok = self.current_tok().clone();
                    self.consume_tok();
                    expr = ASTNode::UnaryExpression {
                        operator: op_tok.lexeme,
                        operand: Box::new(expr),
                        is_postfix: true,
                    }
                }
                TN::LSqu => {
                    self.consume_tok(); // consume '['
                    let index_expr = self.parse_expression()?;
                    self.expect_tok(TN::RSqu)?;
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
            TN::MinusMinus | TN::PlusPlus | TN::Minus | TN::Not => {
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
            TN::Nothing => {
                self.consume_tok();
                Ok(ASTNode::UnitLiteral)
            }
            TN::Number => {
                self.consume_tok();
                Ok(ASTNode::NumberLiteral(tok.lexeme))
            }
            TN::True | TN::False => {
                self.consume_tok();
                Ok(ASTNode::BoolLiteral(tok.lexeme))
            }
            TN::StringLiteral => {
                self.consume_tok();
                Ok(ASTNode::StringLiteral(tok.lexeme))
            }
            // Check for possible function call
            TN::Identifier => {
                let name = self.expect_tok(TN::Identifier)?.lexeme;
                if self.current_tok().tok_name == TN::LPar {
                    self.consume_tok();
                    let mut arguments = Vec::new();
                    if self.current_tok().tok_name != TN::RPar {
                        loop {
                            let expr_arg = self.parse_expression()?;
                            arguments.push(expr_arg);

                            if self.current_tok().tok_name == TN::Comma {
                                self.consume_tok();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect_tok(TN::RPar)?;
                    Ok(ASTNode::FunctionCall {
                        callee: Box::new(ASTNode::Identifier(name)),
                        arguments,
                    })
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            }
            TN::LPar => {
                self.consume_tok();
                let expr = self.parse_expression()?;
                self.expect_tok(TN::RPar)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", tok)),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current_tok().tok_name {
            TN::Int => {
                self.consume_tok();
                Ok(Type::Int)
            }
            TN::Float => {
                self.consume_tok();
                Ok(Type::Float)
            }
            TN::Bool => {
                self.consume_tok();
                Ok(Type::Bool)
            }
            TN::String => {
                self.consume_tok();
                Ok(Type::String)
            }
            TN::Nothing => {
                self.consume_tok();
                Ok(Type::Unit)
            }
            TN::Identifier => {
                let id = self.current_tok().lexeme.clone();
                self.consume_tok();
                Ok(Type::Custom(id))
            }
            _ => Err("Expected a type".to_string()),
        }
    }
}
