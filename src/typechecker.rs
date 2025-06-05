use std::collections::HashMap;

use crate::types::{ASTNode, Type};

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    current_expected_return: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            current_expected_return: None,
            scopes: vec![HashMap::new()],
        }
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

    pub fn check_program(&mut self, nodes: &[ASTNode]) -> Result<(), String> {
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
    fn check(&mut self, node: &ASTNode) -> Result<(Type, bool), String> {
        match node {
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                let expr_ty = self.check(expr)?;
                if let Some(declared_ty) = var_type {
                    if *declared_ty != expr_ty.0 {
                        return Err(format!(
                            "Type mismatch in Let Declaration: variable '{}' declared as {:?} but expression has type {:?}",
                            identifier, var_type, expr_ty
                        ));
                    }
                    self.insert(identifier.clone(), declared_ty.clone());
                    Ok((declared_ty.clone(), false))
                } else {
                    self.insert(identifier.clone(), expr_ty.0.clone());
                    Ok((expr_ty.0, false))
                }
            }
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
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
                    return Err(format!(
                        "Function '{}' does not guarantee a return on all paths",
                        name
                    ));
                }

                Ok((func_ty, false))

                // TODO: The expected return type is not checked in the body
            }
            ASTNode::Body { nodes } => {
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
            } => {
                let guard_ty = self.check(guard)?;

                if guard_ty.0 != Type::Bool {
                    return Err(format!(
                        "Guard in If statement should be of type Bool, but was {:?}",
                        guard_ty.0,
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
            ASTNode::While { guard, body } => {
                let guard_ty = self.check(guard)?;
                if guard_ty.0 != Type::Bool {
                    return Err(format!(
                        "Guard in While statement should be of type Bool, but was {:?}",
                        guard_ty.0,
                    ));
                }
                let (_body_ty, _body_returns) = self.check(body)?;
                // loops may or may not return; conservatively assume no guaranteed return after while
                Ok((Type::Unit, false))
            }
            ASTNode::Break => Ok((Type::Unit, false)),
            ASTNode::FunctionCall { callee, arguments } => {
                let (callee_ty, _callee_ret) = self.check(callee)?;

                match callee_ty {
                    Type::Function {
                        param_types,
                        return_type,
                    } => {
                        // Check args count
                        if param_types.len() != arguments.len() {
                            return Err(format!(
                                "Function called with wrong number of arguments: expected {}, found {}",
                                param_types.len(),
                                arguments.len()
                            ));
                        }

                        for (arg_node, param_ty) in arguments.iter().zip(param_types.iter()) {
                            let arg_ty = self.check(arg_node)?;
                            if *param_ty != arg_ty.0 {
                                return Err(format!(
                                    "Function argument type mismatch: expected {:?}, found {:?}",
                                    param_ty, arg_ty
                                ));
                            }
                        }

                        Ok((*return_type.clone(), false))
                    }
                    other => Err(format!("Attempted to call non-function type {:?}", other)),
                }
            }
            ASTNode::UnitLiteral => Ok((Type::Unit, false)),
            // FIX: how to handle floats?
            ASTNode::NumberLiteral(_) => Ok((Type::Int, false)),
            ASTNode::StringLiteral(_) => Ok((Type::String, false)),
            ASTNode::BoolLiteral(_) => Ok((Type::Bool, false)),
            ASTNode::Identifier(name) => {
                if let Some(t) = self.get(name.to_string()) {
                    Ok((t.clone(), false))
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            ASTNode::Return(expr) => {
                let expr_ty = self.check(expr)?;
                if let Some(expected) = &self.current_expected_return {
                    if expr_ty.0 != *expected {
                        return Err(format!(
                            "Return type does not match expected type in function signature: Expected {:?}, returned {:?}",
                            expected, expr_ty.0
                        ));
                    }
                }
                Ok((expr_ty.0, true))
            }
            ASTNode::ExpressionStatement(expr) => {
                let _ = self.check(expr)?;
                Ok((Type::Unit, false))
            }
            ASTNode::UnaryExpression {
                operator,
                operand,
                is_postfix: _,
            } => {
                let (operand_ty, _operand_ret) = self.check(operand)?;

                match operator.as_str() {
                    "!" => {
                        if operand_ty == Type::Bool {
                            Ok((Type::Bool, false))
                        } else {
                            Err(format!(
                                "Unary operator '{}' requires Bool operand but found {:?}",
                                operator, operand_ty,
                            ))
                        }
                    }
                    "-" | "+" | "--" | "++" => {
                        if operand_ty == Type::Int {
                            Ok((Type::Int, false))
                        } else {
                            Err(format!(
                                "Unary operator '{}' requires Integer operand but found {:?}",
                                operator, operand_ty,
                            ))
                        }
                    }
                    _ => Err(format!("Unknown unary operator '{}'", operator)),
                }
            }
            ASTNode::BinaryExpression {
                left,
                operator,
                right,
            } => {
                let (left_ty, _left_ret) = self.check(left)?;
                let (right_ty, _right_ret) = self.check(right)?;

                match operator.as_str() {
                    ">" | "<" | ">=" | "<=" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok((Type::Bool, false))
                        } else {
                            Err(format!(
                                "Operator '{}' requires Integer operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok((Type::Int, false))
                        } else {
                            Err(format!(
                                "Operator '{}' requires Integer operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "&&" | "||" => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Ok((Type::Bool, false))
                        } else {
                            Err(format!(
                                "Operator '{}' requires Boolean operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "==" | "!=" => {
                        if left_ty == right_ty {
                            Ok((Type::Bool, false))
                        } else {
                            Err(format!(
                                "Cannot compare different types. Found left: {:?}, right: {:?}",
                                left_ty, right_ty
                            ))
                        }
                    }
                    _ => Err(format!("Unknown operator '{}'", operator)),
                }
            }
            _ => Err(format!("Type checking not implemented for node {:?}", node)),
        }
    }
}
