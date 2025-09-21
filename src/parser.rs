use crate::error::{ErrorPhase, FlavorError};
use crate::types::{ASTNode, Span, Token, TokenName as TN, Type};

type ParseProduction = Result<ASTNode, FlavorError>;

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

    fn expect_tok(&mut self, expected: TN) -> Result<Token, FlavorError> {
        let tok = self.current_tok();
        if tok.tok_name == expected {
            let tok = tok.clone();
            self.consume_tok();
            Ok(tok)
        } else {
            Err(FlavorError::with_span(
                format!(
                    "Expected token {:?}, found {:?} ('{}')",
                    expected, tok.tok_name, tok.lexeme
                ),
                tok.span,
                ErrorPhase::Parsing,
            ))
        }
    }

    fn get_precedence(token: &Token) -> Option<u8> {
        match token.tok_name {
            TN::Assign => Some(10),
            TN::Times | TN::Div | TN::Percent => Some(150),
            TN::Plus | TN::Minus => Some(120),
            TN::Gt | TN::Lt | TN::Ge | TN::Le => Some(100),
            TN::Eq | TN::NotEq => Some(80),
            TN::And => Some(50),
            TN::Or => Some(40),
            _ => None,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<ASTNode>, FlavorError> {
        let mut nodes = Vec::new();
        while self.current_tok().tok_name != TN::Eof {
            nodes.push(self.parse_statement()?);
        }
        Ok(nodes)
    }

    fn parse_statement(&mut self) -> ParseProduction {
        match self.current_tok().tok_name {
            TN::Print => self.parse_print_statement(),
            TN::Let => self.parse_let_statement(),
            TN::Fn => self.parse_function_declaration(),
            TN::If => self.parse_if(),
            TN::While => self.parse_while(),
            TN::Return => self.parse_return(),
            TN::Break => self.parse_break(),
            TN::LBra => self.parse_body(),
            TN::Identifier => self.parse_expression_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_print_statement(&mut self) -> ParseProduction {
        let print_tok = self.expect_tok(TN::Print)?;
        let mut span = print_tok.span;
        let mut expressions = vec![];
        while self.current_tok().tok_name != TN::Semicolon {
            let expr = self.parse_expression()?;
            span = span.merge(expr.span());
            expressions.push(expr);
            if self.current_tok().tok_name == TN::Comma {
                span = span.merge(self.expect_tok(TN::Comma)?.span);
            } else {
                break;
            }
        }
        let semicolon = self.expect_tok(TN::Semicolon)?;
        span = span.merge(semicolon.span);
        Ok(ASTNode::Print { expressions, span })
    }

    fn parse_while(&mut self) -> ParseProduction {
        let while_tok = self.expect_tok(TN::While)?;
        let guard = self.parse_expression()?;
        let body = self.parse_body()?;

        let span = while_tok.span.merge(guard.span()).merge(body.span());

        Ok(ASTNode::While {
            guard: Box::new(guard),
            body: Box::new(body),
            span,
        })
    }

    fn parse_if(&mut self) -> ParseProduction {
        let if_tok = self.expect_tok(TN::If)?;
        let guard = self.parse_expression()?;
        let then_body = self.parse_body()?;
        let mut span = if_tok.span.merge(guard.span()).merge(then_body.span());

        let mut else_body = None;
        if self.current_tok().tok_name == TN::Else {
            let else_tok = self.expect_tok(TN::Else)?;
            let parsed_else = self.parse_body()?;
            span = span.merge(else_tok.span).merge(parsed_else.span());
            else_body = Some(Box::new(parsed_else));
        }

        Ok(ASTNode::If {
            guard: Box::new(guard),
            then_body: Box::new(then_body),
            else_body,
            span,
        })
    }

    fn parse_break(&mut self) -> ParseProduction {
        let break_tok = self.expect_tok(TN::Break)?;
        let semicolon = self.expect_tok(TN::Semicolon)?;
        let span = break_tok.span.merge(semicolon.span);
        Ok(ASTNode::Break { span })
    }

    fn parse_return(&mut self) -> ParseProduction {
        let return_tok = self.expect_tok(TN::Return)?;
        let mut span = return_tok.span;
        let mut expr = ASTNode::UnitLiteral {
            span: return_tok.span,
        };
        if self.current_tok().tok_name != TN::Semicolon {
            expr = self.parse_expression()?;
            span = span.merge(expr.span());
        }
        let semicolon = self.expect_tok(TN::Semicolon)?;
        span = span.merge(semicolon.span);
        Ok(ASTNode::Return {
            expr: Box::new(expr),
            span,
        })
    }

    fn parse_function_declaration(&mut self) -> ParseProduction {
        let fn_tok = self.expect_tok(TN::Fn)?;
        let fn_name = self.expect_tok(TN::Identifier)?;

        let (parameters, params_span) = self.parse_fn_parameters()?;

        let arrow_tok = self.expect_tok(TN::SlimArrow)?;

        let (return_ty, return_span) = self.parse_type()?;

        let body = self.parse_body()?;

        let span = fn_tok
            .span
            .merge(fn_name.span)
            .merge(params_span)
            .merge(arrow_tok.span)
            .merge(return_span)
            .merge(body.span());

        Ok(ASTNode::FunctionDeclaration {
            name: fn_name.lexeme,
            parameters,
            return_type: return_ty,
            body: Box::new(body),
            span,
        })
    }

    fn parse_body(&mut self) -> Result<ASTNode, FlavorError> {
        let lbra = self.expect_tok(TN::LBra)?;

        let mut statements: Vec<ASTNode> = vec![];
        let mut span = lbra.span;
        while self.current_tok().tok_name != TN::RBra {
            let statement = self.parse_statement()?;
            span = span.merge(statement.span());
            statements.push(statement);
        }
        let rbra = self.expect_tok(TN::RBra)?;
        span = span.merge(rbra.span);
        Ok(ASTNode::Body {
            nodes: statements,
            span,
        })
    }

    fn parse_fn_parameters(&mut self) -> Result<(Vec<(String, Type)>, Span), FlavorError> {
        let lpar = self.expect_tok(TN::LPar)?;
        let mut span = lpar.span;
        let mut params: Vec<(String, Type)> = vec![];
        if self.current_tok().tok_name != TN::RPar {
            loop {
                let param_name = self.expect_tok(TN::Identifier)?;
                span = span.merge(param_name.span);
                let colon_tok = self.expect_tok(TN::Colon)?;
                span = span.merge(colon_tok.span);
                let (param_ty, ty_span) = self.parse_type()?;
                span = span.merge(ty_span);
                params.push((param_name.lexeme, param_ty));
                if self.current_tok().tok_name == TN::Comma {
                    let comma = self.expect_tok(TN::Comma)?;
                    span = span.merge(comma.span);
                } else {
                    break;
                }
            }
        }

        let rpar = self.expect_tok(TN::RPar)?;
        span = span.merge(rpar.span);

        Ok((params, span))
    }

    fn parse_function_type_signature(&mut self) -> Result<(Type, Span), FlavorError> {
        let lpar = self.expect_tok(TN::LPar)?;
        let mut span = lpar.span;
        let mut param_types: Vec<Type> = Vec::new();
        if self.current_tok().tok_name != TN::RPar {
            loop {
                let (param_type, ty_span) = self.parse_type()?;
                span = span.merge(ty_span);
                param_types.push(param_type);
                if self.current_tok().tok_name == TN::Comma {
                    let comma = self.expect_tok(TN::Comma)?;
                    span = span.merge(comma.span);
                } else {
                    break;
                }
            }
        }
        let rpar = self.expect_tok(TN::RPar)?;
        span = span.merge(rpar.span);
        let arrow_tok = self.expect_tok(TN::SlimArrow)?;
        span = span.merge(arrow_tok.span);
        let (return_type, return_span) = self.parse_type()?;
        span = span.merge(return_span);
        Ok((
            Type::Function {
                param_types,
                return_type: Box::new(return_type),
            },
            span,
        ))
    }

    fn parse_let_statement(&mut self) -> ParseProduction {
        let let_tok = self.expect_tok(TN::Let)?;
        let mut span = let_tok.span;
        let id_tok = self.expect_tok(TN::Identifier)?;
        span = span.merge(id_tok.span);

        let var_type: Option<Type> = if self.current_tok().tok_name == TN::Colon {
            let colon_tok = self.expect_tok(TN::Colon)?;
            span = span.merge(colon_tok.span);
            let (ty, ty_span) = self.parse_type()?;
            span = span.merge(ty_span);
            Some(ty)
        } else {
            None
        };

        let assign_tok = self.expect_tok(TN::Assign)?;
        span = span.merge(assign_tok.span);
        let expr = self.parse_expression()?;
        span = span.merge(expr.span());
        let semicolon = self.expect_tok(TN::Semicolon)?;
        span = span.merge(semicolon.span);

        Ok(ASTNode::LetDeclaration {
            identifier: id_tok.lexeme,
            var_type,
            expr: Box::new(expr),
            span,
        })
    }

    fn parse_expression_statement(&mut self) -> ParseProduction {
        let expr = self.parse_expression()?;
        let semicolon = self.expect_tok(TN::Semicolon)?;
        let span = expr.span().merge(semicolon.span);
        Ok(ASTNode::ExpressionStatement {
            expr: Box::new(expr),
            span,
        })
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
            let span = left.span().merge(op_tok.span).merge(right.span());
            left = ASTNode::BinaryExpression {
                left: Box::new(left),
                operator: op_tok.lexeme,
                right: Box::new(right),
                span,
            }
        }
        Ok(left)
    }

    fn parse_postfix_expression(&mut self) -> ParseProduction {
        let mut expr = self.parse_unary_expression()?;

        loop {
            match self.current_tok().tok_name {
                TN::PlusPlus | TN::MinusMinus => {
                    let op_tok = self.current_tok().clone();
                    self.consume_tok();
                    let span = expr.span().merge(op_tok.span);
                    expr = ASTNode::UnaryExpression {
                        operator: op_tok.lexeme,
                        operand: Box::new(expr),
                        is_postfix: true,
                        span,
                    };
                }
                TN::LSqu => {
                    let bracket_tok = self.expect_tok(TN::LSqu)?;
                    let index_expr = self.parse_expression()?;
                    let close_tok = self.expect_tok(TN::RSqu)?;
                    let span = expr
                        .span()
                        .merge(bracket_tok.span)
                        .merge(index_expr.span())
                        .merge(close_tok.span);
                    expr = ASTNode::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index_expr),
                        span,
                    };
                }
                TN::LPar => {
                    let lpar = self.expect_tok(TN::LPar)?;
                    let mut span = expr.span().merge(lpar.span);
                    let mut arguments = Vec::new();
                    if self.current_tok().tok_name != TN::RPar {
                        loop {
                            let argument = self.parse_expression()?;
                            span = span.merge(argument.span());
                            arguments.push(argument);
                            if self.current_tok().tok_name == TN::Comma {
                                span = span.merge(self.expect_tok(TN::Comma)?.span);
                            } else {
                                break;
                            }
                        }
                    }
                    let rpar = self.expect_tok(TN::RPar)?;
                    span = span.merge(rpar.span);
                    expr = ASTNode::FunctionCall {
                        callee: Box::new(expr),
                        arguments,
                        span,
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
                self.consume_tok();
                let operand = self.parse_unary_expression()?;
                let span = tok.span.merge(operand.span());
                Ok(ASTNode::UnaryExpression {
                    operator: tok.lexeme,
                    operand: Box::new(operand),
                    is_postfix: false,
                    span,
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> ParseProduction {
        let tok = self.current_tok().clone();
        match tok.tok_name {
            TN::Nothing => {
                let tok = self.expect_tok(TN::Nothing)?;
                Ok(ASTNode::UnitLiteral { span: tok.span })
            }
            TN::Number => {
                let tok = self.expect_tok(TN::Number)?;
                Ok(ASTNode::NumberLiteral {
                    value: tok.lexeme,
                    span: tok.span,
                })
            }
            TN::True | TN::False => {
                let tok_name = tok.tok_name.clone();
                let tok = match tok_name {
                    TN::True => self.expect_tok(TN::True)?,
                    TN::False => self.expect_tok(TN::False)?,
                    _ => unreachable!(),
                };
                Ok(ASTNode::BoolLiteral {
                    value: tok.lexeme,
                    span: tok.span,
                })
            }
            TN::StringLiteral => {
                let tok = self.expect_tok(TN::StringLiteral)?;
                Ok(ASTNode::StringLiteral {
                    value: tok.lexeme,
                    span: tok.span,
                })
            }
            TN::Fn => {
                let fn_tok = self.expect_tok(TN::Fn)?;
                let (parameters, params_span) = self.parse_fn_parameters()?;
                let arrow_tok = self.expect_tok(TN::SlimArrow)?;
                let (return_ty, return_span) = self.parse_type()?;
                let body = self.parse_body()?;
                let span = fn_tok
                    .span
                    .merge(params_span)
                    .merge(arrow_tok.span)
                    .merge(return_span)
                    .merge(body.span());
                Ok(ASTNode::FunctionExpression {
                    parameters,
                    return_type: return_ty,
                    body: Box::new(body),
                    span,
                })
            }
            TN::LSqu => {
                let lsqu = self.expect_tok(TN::LSqu)?;
                let mut span = lsqu.span;
                let mut elements = Vec::new();
                if self.current_tok().tok_name != TN::RSqu {
                    loop {
                        let elem = self.parse_expression()?;
                        span = span.merge(elem.span());
                        elements.push(elem);
                        if self.current_tok().tok_name == TN::Comma {
                            span = span.merge(self.expect_tok(TN::Comma)?.span);
                        } else {
                            break;
                        }
                    }
                }
                let rsqu = self.expect_tok(TN::RSqu)?;
                span = span.merge(rsqu.span);
                Ok(ASTNode::ArrayLiteral { elements, span })
            }
            TN::Identifier => {
                let tok = self.expect_tok(TN::Identifier)?;
                Ok(ASTNode::Identifier {
                    name: tok.lexeme,
                    span: tok.span,
                })
            }
            TN::LPar => {
                self.expect_tok(TN::LPar)?;
                let expr = self.parse_expression()?;
                self.expect_tok(TN::RPar)?;
                Ok(expr)
            }
            _ => Err(FlavorError::with_span(
                format!("Unexpected token in expression: {tok:?}"),
                tok.span,
                ErrorPhase::Parsing,
            )),
        }
    }

    fn parse_type(&mut self) -> Result<(Type, Span), FlavorError> {
        match self.current_tok().tok_name {
            TN::Int => {
                let tok = self.expect_tok(TN::Int)?;
                Ok((Type::Int, tok.span))
            }
            TN::Float => {
                let tok = self.expect_tok(TN::Float)?;
                Ok((Type::Float, tok.span))
            }
            TN::Bool => {
                let tok = self.expect_tok(TN::Bool)?;
                Ok((Type::Bool, tok.span))
            }
            TN::String => {
                let tok = self.expect_tok(TN::String)?;
                Ok((Type::String, tok.span))
            }
            TN::Nothing => {
                let tok = self.expect_tok(TN::Nothing)?;
                Ok((Type::Unit, tok.span))
            }
            TN::Identifier => {
                let tok = self.expect_tok(TN::Identifier)?;
                Ok((Type::Custom(tok.lexeme), tok.span))
            }
            TN::Array => {
                let array_tok = self.expect_tok(TN::Array)?;
                let lpar = self.expect_tok(TN::LPar)?;
                let (element_type, element_span) = self.parse_type()?;
                let rpar = self.expect_tok(TN::RPar)?;
                let span = array_tok
                    .span
                    .merge(lpar.span)
                    .merge(element_span)
                    .merge(rpar.span);
                Ok((Type::Array(Box::new(element_type)), span))
            }
            TN::LPar => self.parse_function_type_signature(),
            _ => Err(FlavorError::with_span(
                "Expected a type",
                self.current_tok().span,
                ErrorPhase::Parsing,
            )),
        }
    }
}
