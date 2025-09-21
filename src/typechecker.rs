use std::collections::HashMap;

use crate::error::{ErrorPhase, FlavorError};
use crate::types::{ASTNode, Type};

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    current_expected_return: Option<Type>,
    current_expected_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            current_expected_return: None,
            current_expected_type: None,
            scopes: vec![HashMap::new()],
        }
    }

    fn with_expected_type<F, R>(&mut self, expected: Option<Type>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let previous = self.current_expected_type.clone();
        self.current_expected_type = expected;
        let result = f(self);
        self.current_expected_type = previous;
        result
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    pub fn get(&self, name: String) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(&name) {
                return Some(ty);
            }
        }
        None
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn check_program(&mut self, nodes: &[ASTNode]) -> Result<(), FlavorError> {
        for n in nodes {
            self.check(n)?;
        }
        Ok(())
    }

    /// Return a Result type
    /// Positive case => (Type, bool) the type is the type of the ASTNode and the bool shows if it
    /// is a return (useful to check all paths)
    /// Negative case => Error String
    ///
    /// * `node`:
    fn check(&mut self, node: &ASTNode) -> Result<(Type, bool), FlavorError> {
        match node {
            ASTNode::Print { expressions, .. } => {
                for expr in expressions.clone().into_iter() {
                    let _ = self.check(&expr)?;
                }
                Ok((Type::Unit, false))
            }
            ASTNode::ArrayLiteral { elements, span } => {
                if elements.is_empty() {
                    if let Some(Type::Array(expected)) = self.current_expected_type.clone() {
                        return Ok((Type::Array(expected), false));
                    }
                    return Ok((Type::Array(Box::new(Type::Unit)), false));
                }

                let mut element_type: Option<Type> = None;

                for elem in elements {
                    let (elem_ty, _returns) = self.check(elem)?;
                    if let Some(ref t) = element_type {
                        if *t != elem_ty {
                            return Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Array elements must be of the same type, found {t:?} and {elem_ty:?}"
                                ),
                                *elem.span(),
                            ));
                        }
                    } else {
                        element_type = Some(elem_ty);
                    }
                }

                let final_elem_type = element_type.unwrap();

                if let Some(Type::Array(expected)) = self.current_expected_type.clone() {
                    if *expected != final_elem_type {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Array literal element type mismatch: expected {:?}, found {:?}",
                                *expected, final_elem_type
                            ),
                            *span,
                        ));
                    }
                    return Ok((Type::Array(expected), false));
                }

                Ok((Type::Array(Box::new(final_elem_type)), false))
            }
            ASTNode::ArrayAccess { array, index, .. } => {
                let (array_ty, _array_ret) = self.check(array)?;
                let (index_ty, _index_ret) = self.check(index)?;

                if index_ty != Type::Int {
                    return Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Array index must be of type Int, found {index_ty:?}",),
                        *index.span(),
                    ));
                }

                match array_ty {
                    Type::Array(elem_type) => Ok((*elem_type, false)),
                    other => Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Attempted to index non-array type {other:?}"),
                        *array.span(),
                    )),
                }
            }
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
                span,
            } => {
                if let ASTNode::FunctionExpression {
                    parameters,
                    return_type,
                    body,
                    ..
                } = expr.as_ref()
                {
                    let inferred_ty = Type::Function {
                        param_types: parameters.iter().map(|(_, ty)| ty.clone()).collect(),
                        return_type: Box::new(return_type.clone()),
                    };

                    if let Some(declared_ty) = var_type {
                        let mut matches = true;
                        match declared_ty {
                            Type::Function {
                                param_types,
                                return_type: declared_return,
                            } => {
                                if param_types.len() != parameters.len() {
                                    matches = false;
                                } else {
                                    for ((_, actual_ty), expected_ty) in
                                        parameters.iter().zip(param_types.iter())
                                    {
                                        if actual_ty != expected_ty {
                                            matches = false;
                                            break;
                                        }
                                    }
                                    if *declared_return.as_ref() != return_type.clone() {
                                        matches = false;
                                    }
                                }
                            }
                            _ => matches = false,
                        }

                        if !matches {
                            return Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Type mismatch in let declaration: variable '{identifier}' declared as {declared_ty:?} but expression has type {inferred_ty:?}"
                                ),
                                *span,
                            ));
                        }
                    }

                    let stored_ty = var_type.clone().unwrap_or(inferred_ty.clone());
                    self.insert(identifier.clone(), stored_ty.clone());

                    let previous_expected_return = self.current_expected_return.clone();
                    let expected_return = if let Type::Function { return_type, .. } = &stored_ty {
                        *return_type.clone()
                    } else {
                        Type::Unit
                    };
                    self.current_expected_return = Some(expected_return.clone());

                    self.enter_scope();
                    for (param_name, param_ty) in parameters {
                        self.insert(param_name.clone(), param_ty.clone());
                    }
                    let (_, guaranteed_return) = self.check(body)?;
                    self.exit_scope();
                    self.current_expected_return = previous_expected_return;

                    if expected_return != Type::Unit && !guaranteed_return {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Function assigned to '{identifier}' does not guarantee a return on all paths"
                            ),
                            *body.span(),
                        ));
                    }

                    return Ok((stored_ty, false));
                }

                let expr_result = if let Some(declared_ty) = var_type {
                    self.with_expected_type(Some(declared_ty.clone()), |tc| tc.check(expr))?
                } else {
                    self.check(expr)?
                };

                if let Some(declared_ty) = var_type {
                    if *declared_ty != expr_result.0 {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Type mismatch in let declaration: variable '{identifier}' declared as {declared_ty:?} but expression has type {:?}",
                                expr_result.0
                            ),
                            *expr.span(),
                        ));
                    }
                    self.insert(identifier.clone(), declared_ty.clone());
                    Ok((declared_ty.clone(), false))
                } else {
                    self.insert(identifier.clone(), expr_result.0.clone());
                    Ok((expr_result.0, false))
                }
            }
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
                span,
            } => {
                let func_ty = Type::Function {
                    param_types: parameters.iter().map(|(_, t)| t.clone()).collect(),
                    return_type: Box::new(return_type.clone()),
                };

                let old_expected = self.current_expected_return.clone();
                self.current_expected_return = Some(return_type.clone());

                self.insert(name.clone(), func_ty.clone());

                // Enter Scope for function Body
                self.enter_scope();

                for (param_name, param_ty) in parameters {
                    self.insert(param_name.clone(), param_ty.clone());
                }

                let (_, guaranteed_ret) = self.check(body)?;

                self.exit_scope();

                self.current_expected_return = old_expected;

                if *return_type != Type::Unit && !guaranteed_ret {
                    return Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Function '{name}' does not guarantee a return on all paths"),
                        *span,
                    ));
                }

                Ok((func_ty, false))
            }
            ASTNode::FunctionExpression {
                parameters,
                return_type,
                body,
                span,
            } => {
                let func_ty = Type::Function {
                    param_types: parameters.iter().map(|(_, t)| t.clone()).collect(),
                    return_type: Box::new(return_type.clone()),
                };

                let mut enforced_return = return_type.clone();
                if let Some(Type::Function {
                    param_types: expected_params,
                    return_type: expected_return,
                }) = self.current_expected_type.clone()
                {
                    if expected_params.len() != parameters.len() {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Function expression parameter count mismatch: expected {}, found {}",
                                expected_params.len(),
                                parameters.len()
                            ),
                            *span,
                        ));
                    }
                    for ((_, actual_ty), expected_ty) in
                        parameters.iter().zip(expected_params.iter())
                    {
                        if actual_ty != expected_ty {
                            return Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Function expression parameter type mismatch: expected {expected_ty:?}, found {actual_ty:?}"
                                ),
                                *span,
                            ));
                        }
                    }
                    if *expected_return.as_ref() != return_type.clone() {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Function expression return type mismatch: expected {expected_return:?}, found {return_type:?}"
                            ),
                            *span,
                        ));
                    }
                    enforced_return = *expected_return.clone();
                }

                let previous_expected_return = self.current_expected_return.clone();
                self.current_expected_return = Some(enforced_return.clone());

                self.enter_scope();
                for (param_name, param_ty) in parameters {
                    self.insert(param_name.clone(), param_ty.clone());
                }
                let (_, guaranteed_ret) = self.check(body)?;
                self.exit_scope();

                self.current_expected_return = previous_expected_return;

                if enforced_return != Type::Unit && !guaranteed_ret {
                    return Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        "Function expression does not guarantee a return on all paths",
                        *body.span(),
                    ));
                }

                Ok((func_ty, false))
            }
            ASTNode::Body { nodes, .. } => {
                let mut guaranteed_return = false;
                let mut last_type = Type::Unit;

                for n in nodes {
                    let (ty, returns) = self.check(n)?;
                    last_type = ty;
                    if returns {
                        guaranteed_return = true;
                        break; // unreachable code after return
                    }
                }

                Ok((last_type, guaranteed_return))
            }
            ASTNode::If {
                guard,
                then_body,
                else_body,
                ..
            } => {
                let guard_ty = self.check(guard)?;

                if guard_ty.0 != Type::Bool {
                    return Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!(
                            "Guard in If statement should be of type Bool, but was {:?}",
                            guard_ty.0,
                        ),
                        *guard.span(),
                    ));
                }

                let (then_ty, then_returns) = self.check(then_body)?;

                if let Some(else_body) = else_body {
                    let (_else_ty, else_returns) = self.check(else_body)?;

                    // Relaxed: Don't require then_ty == else_ty here.
                    // Instead, only require both branches guarantee return.
                    if then_returns && else_returns {
                        // Both branches return, so the if expression guarantees return
                        // Return the function expected type or some common type if you can
                        Ok((
                            self.current_expected_return.clone().unwrap_or(Type::Unit),
                            true,
                        ))
                    } else {
                        // One or both branches do not guarantee return
                        Ok((then_ty, false))
                    }
                } else {
                    // No else branch means no guaranteed return
                    Ok((then_ty, false))
                }
            }
            ASTNode::While { guard, body, .. } => {
                let guard_ty = self.check(guard)?;
                if guard_ty.0 != Type::Bool {
                    return Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!(
                            "Guard in While statement should be of type Bool, but was {:?}",
                            guard_ty.0,
                        ),
                        *guard.span(),
                    ));
                }
                let (_body_ty, _body_returns) = self.check(body)?;
                // loops may or may not return; conservatively assume no guaranteed return after while
                Ok((Type::Unit, false))
            }
            ASTNode::Break { .. } => Ok((Type::Unit, false)),
            ASTNode::FunctionCall {
                callee,
                arguments,
                span,
            } => {
                let (callee_ty, _callee_ret) = self.check(callee)?;

                match callee_ty {
                    Type::Function {
                        param_types,
                        return_type,
                    } => {
                        // Check args count
                        if param_types.len() != arguments.len() {
                            return Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Function argument count mismatch: expected {}, found {}",
                                    param_types.len(),
                                    arguments.len()
                                ),
                                *span,
                            ));
                        }

                        for (arg_node, param_ty) in arguments.iter().zip(param_types.iter()) {
                            let arg_ty = self.with_expected_type(Some(param_ty.clone()), |tc| {
                                tc.check(arg_node)
                            })?;
                            if *param_ty != arg_ty.0 {
                                return Err(FlavorError::with_span(
                                    ErrorPhase::TypeChecking,
                                    format!(
                                        "Function argument type mismatch: expected {param_ty:?}, found {:?}",
                                        arg_ty.0
                                    ),
                                    *arg_node.span(),
                                ));
                            }
                        }

                        Ok((*return_type.clone(), false))
                    }
                    other => Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Attempted to call non-function type {other:?}"),
                        *span,
                    )),
                }
            }
            ASTNode::UnitLiteral { .. } => Ok((Type::Unit, false)),
            ASTNode::NumberLiteral { .. } => Ok((Type::Int, false)),
            ASTNode::StringLiteral { .. } => Ok((Type::String, false)),
            ASTNode::BoolLiteral { .. } => Ok((Type::Bool, false)),
            ASTNode::Identifier { name, span } => {
                if let Some(t) = self.get(name.to_string()) {
                    Ok((t.clone(), false))
                } else {
                    Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Use of undeclared identifier '{name}'"),
                        *span,
                    ))
                }
            }
            ASTNode::Return { expr, span } => {
                let expected_return = self.current_expected_return.clone();
                let expr_ty = if expected_return.is_some() {
                    self.with_expected_type(expected_return.clone(), |tc| tc.check(expr))?
                } else {
                    self.check(expr)?
                };
                if let Some(expected) = &expected_return {
                    if expr_ty.0 != *expected {
                        return Err(FlavorError::with_span(
                            ErrorPhase::TypeChecking,
                            format!(
                                "Return type mismatch: expected {expected:?}, found {:?}",
                                expr_ty.0
                            ),
                            *span,
                        ));
                    }
                }
                Ok((expr_ty.0, true))
            }
            ASTNode::ExpressionStatement { expr, .. } => {
                let _ = self.check(expr)?;
                Ok((Type::Unit, false))
            }
            ASTNode::UnaryExpression {
                operator,
                operand,
                is_postfix: _,
                span,
            } => {
                let (operand_ty, _operand_ret) = self.check(operand)?;

                match operator.as_str() {
                    "!" => {
                        if operand_ty == Type::Bool {
                            Ok((Type::Bool, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Unary operator '{operator}' requires Boolean operand but found {operand_ty:?}",
                                ),
                                *span,
                            ))
                        }
                    }
                    "-" | "+" | "--" | "++" => {
                        if operand_ty == Type::Int {
                            Ok((Type::Int, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Unary operator '{operator}' requires Integer operand but found {operand_ty:?}",
                                ),
                                *span,
                            ))
                        }
                    }
                    _ => Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Unknown unary operator '{operator}'"),
                        *span,
                    )),
                }
            }
            ASTNode::BinaryExpression {
                left,
                operator,
                right,
                span,
            } => {
                let (left_ty, _left_ret) = self.check(left)?;
                let (right_ty, _right_ret) = if operator == "=" {
                    self.with_expected_type(Some(left_ty.clone()), |tc| tc.check(right))?
                } else {
                    self.check(right)?
                };

                match operator.as_str() {
                    "=" => {
                        if left_ty != right_ty {
                            return Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Assignment type mismatch: left is {left_ty:?}, right is {right_ty:?}"
                                ),
                                *span,
                            ));
                        }
                        Ok((left_ty, false))
                    }
                    ">" | "<" | ">=" | "<=" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok((Type::Bool, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Operator '{operator}' requires Integer operands but found left: {left_ty:?}, right: {right_ty:?}"
                                ),
                                *span,
                            ))
                        }
                    }
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok((Type::Int, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Operator '{operator}' requires Integer operands but found left: {left_ty:?}, right: {right_ty:?}"
                                ),
                                *span,
                            ))
                        }
                    }
                    "%" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok((Type::Int, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Operator '{operator}' requires Integer operands but found left: {left_ty:?}, right: {right_ty:?}"
                                ),
                                *span,
                            ))
                        }
                    }
                    "&&" | "||" => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Ok((Type::Bool, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Operator '{operator}' requires Boolean operands but found left: {left_ty:?}, right: {right_ty:?}"
                                ),
                                *span,
                            ))
                        }
                    }
                    "==" | "!=" => {
                        if left_ty == right_ty {
                            Ok((Type::Bool, false))
                        } else {
                            Err(FlavorError::with_span(
                                ErrorPhase::TypeChecking,
                                format!(
                                    "Operator '{operator}' requires operands of the same type but found left: {left_ty:?}, right: {right_ty:?}"
                                ),
                                *span,
                            ))
                        }
                    }
                    _ => Err(FlavorError::with_span(
                        ErrorPhase::TypeChecking,
                        format!("Unknown binary operator '{operator}'"),
                        *span,
                    )),
                }
            }
        }
    }
}
