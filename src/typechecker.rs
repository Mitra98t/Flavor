use std::collections::HashMap;

use crate::types::{ASTNode, Type};

pub struct TypeChecker {
    variables: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.variables.insert(name, ty);
    }

    pub fn get(&self, name: String) -> Option<&Type> {
        self.variables.get(&name)
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
            // FIX: how to handle floats?
            ASTNode::NumberLiteral(_) => Ok(Type::Int),
            ASTNode::Identifier(name) => {
                if let Some(t) = self.get(name.to_string()) {
                    Ok(t.clone())
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            ASTNode::ExpressionStatement(expr) => {
                let _ = self.check(expr);
                Ok(Type::Unit)
            }
            ASTNode::BinaryExpression {
                left,
                operator,
                right,
            } => {
                let left_ty = self.check(left)?;
                let right_ty = self.check(right)?;

                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!(
                                "Operator '{}' requires Integer operands but found left: {:?}, right: {:?}",
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
