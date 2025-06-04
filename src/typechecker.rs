use std::collections::HashMap;

use crate::types::{ASTNode, Type};

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
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

    fn check(&mut self, node: &ASTNode) -> Result<Type, String> {
        match node {
            ASTNode::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                let expr_ty = self.check(expr)?;
                if let Some(declared_ty) = var_type {
                    if *declared_ty != expr_ty {
                        return Err(format!(
                            "Type mismatch in Let Declaration: variable '{}' declared as {:?} but expression has type {:?}",
                            identifier, var_type, expr_ty
                        ));
                    }
                    self.insert(identifier.clone(), declared_ty.clone());
                    Ok(declared_ty.clone())
                } else {
                    self.insert(identifier.clone(), expr_ty.clone());
                    Ok(expr_ty)
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

                self.insert(name.clone(), func_ty);

                // Enter Scope for function Body
                self.enter_scope();

                for (param_name, param_ty) in parameters {
                    self.insert(param_name.clone(), param_ty.clone());
                }

                let _ = self.check(body)?;

                self.exit_scope();
                Ok(return_type.clone())

                // TODO: The expected return type is not checked in the body
            }
            ASTNode::Body { nodes } => {
                for n in nodes {
                    self.check(n)?;
                }
                Ok(Type::Unit)
            }
            ASTNode::If {
                guard,
                then_body,
                else_body,
            } => {
                let guard_ty = self.check(guard)?;

                if guard_ty != Type::Bool {
                    return Err(format!(
                        "Guard in If statement should be of type Bool, but was {:?}",
                        guard_ty,
                    ));
                }

                let _ = self.check(then_body)?;
                if let Some(else_body) = else_body {
                    let _ = self.check(else_body)?;
                }
                Ok(Type::Unit)
            }
            ASTNode::While { guard, body } => {
                let guard_ty = self.check(guard)?;

                if guard_ty != Type::Bool {
                    return Err(format!(
                        "Guard in While statement should be of type Bool, but was {:?}",
                        guard_ty,
                    ));
                }

                let _ = self.check(body)?;
                Ok(Type::Unit)
            }
            ASTNode::Break => Ok(Type::Unit),
            ASTNode::FunctionCall { callee, arguments } => {
                let callee_ty = self.check(callee)?;

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
                            if *param_ty != arg_ty {
                                return Err(format!(
                                    "Function argument type mismatch: expected {:?}, found {:?}",
                                    param_ty, arg_ty
                                ));
                            }
                        }

                        Ok(*return_type.clone())
                    }
                    other => Err(format!("Attempted to call non-function type {:?}", other)),
                }
            }
            ASTNode::UnitLiteral => Ok(Type::Unit),
            // FIX: how to handle floats?
            ASTNode::NumberLiteral(_) => Ok(Type::Int),
            ASTNode::StringLiteral(_) => Ok(Type::String),
            ASTNode::BoolLiteral(_) => Ok(Type::Bool),
            ASTNode::Identifier(name) => {
                if let Some(t) = self.get(name.to_string()) {
                    Ok(t.clone())
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            ASTNode::Return(expr) => {
                let expr_ty = self.check(expr)?;
                Ok(expr_ty)
            }
            ASTNode::ExpressionStatement(expr) => {
                let _ = self.check(expr)?;
                Ok(Type::Unit)
            }
            ASTNode::UnaryExpression {
                operator,
                operand,
                is_postfix: _,
            } => {
                let operand_ty = self.check(operand)?;

                match operator.as_str() {
                    "!" => {
                        if operand_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Unary operator '{}' requires Bool operand but found {:?}",
                                operator, operand_ty,
                            ))
                        }
                    }
                    "-" | "+" | "--" | "++" => {
                        if operand_ty == Type::Int {
                            Ok(Type::Int)
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
                let left_ty = self.check(left)?;
                let right_ty = self.check(right)?;

                match operator.as_str() {
                    "+" | "-" | "*" | "/" | ">" | "<" | ">=" | "<=" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!(
                                "Operator '{}' requires Integer operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "&&" | "||" => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Operator '{}' requires Boolean operands but found left: {:?}, right: {:?}",
                                operator, left_ty, right_ty
                            ))
                        }
                    }
                    "==" | "!=" => {
                        if left_ty == right_ty {
                            Ok(Type::Bool)
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
