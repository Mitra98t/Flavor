use std::collections::HashMap;

use crate::types::ASTNode as AST;

#[derive(Debug, Clone)]
pub enum EvaluationType {
    Int(i64),
    // Float(f64),
    Bool(bool),
    #[allow(dead_code)]
    String(String),
    Unit,
    Array(Vec<EvaluationType>),
    Function {
        parameters: Vec<String>,
        body: Box<AST>,
        env: HashMap<String, EvaluationType>,
    },
}

impl EvaluationType {}

pub struct Interpreter {
    pub env: HashMap<String, EvaluationType>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: HashMap::new(),
        }
    }

    pub fn eval_program(&mut self, nodes: &[AST]) -> Result<Vec<EvaluationType>, String> {
        let mut result = vec![];
        for node in nodes {
            result.push(self.eval(node)?);
        }
        Ok(result)
    }

    fn eval(&mut self, node: &AST) -> Result<EvaluationType, String> {
        match node {
            AST::Print(exprs) => {
                let mut outputs = Vec::new();
                for expr in exprs.iter() {
                    let eval = self.eval(expr)?;
                    outputs.push(format!("{eval:?}"));
                }
                println!("{}", outputs.join(" "));
                Ok(EvaluationType::Unit)
            }
            AST::Body { nodes } => {
                let mut result = EvaluationType::Unit;
                for n in nodes {
                    result = self.eval(n)?;
                }
                Ok(result)
            }
            AST::If {
                guard,
                then_body,
                else_body,
            } => {
                let guard_value = self.eval(guard)?;

                if let EvaluationType::Bool(true) = guard_value {
                    self.eval(then_body)
                } else if let Some(else_body) = else_body {
                    self.eval(else_body)
                } else {
                    Ok(EvaluationType::Unit)
                }
            }
            AST::While { guard, body } => {
                let mut result = EvaluationType::Unit;
                while let EvaluationType::Bool(true) = self.eval(guard)? {
                    result = self.eval(body)?;
                    // handle break statement
                }
                Ok(result)
            }
            AST::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                let value = self.eval(expr)?;
                if let Some(_var_type) = var_type {
                    // TODO: Type checking?
                }
                self.env.insert(identifier.clone(), value);
                Ok(EvaluationType::Unit)
            }
            AST::FunctionDeclaration {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let func = EvaluationType::Function {
                    parameters: parameters.clone().iter().map(|p| p.clone().0).collect(),
                    body: Box::new(*body.clone()),
                    env: self.env.clone(),
                };
                self.env.insert(name.clone(), func);
                Ok(EvaluationType::Unit)
            }
            AST::Return(expr) => {
                let value = self.eval(expr)?;
                Ok(value)
            }
            AST::Break => {
                // Handle break logic, if needed
                Ok(EvaluationType::Unit)
            }
            AST::FunctionCall { callee, arguments } => {
                // TODO: rework error handling
                let func = match self.eval(callee)? {
                    EvaluationType::Function {
                        parameters,
                        body,
                        env,
                    } => (parameters, body, env),
                    _ => return Err("Callee is not a function".to_string()),
                };

                if func.0.len() != arguments.len() {
                    return Err("Argument count mismatch".to_string());
                }

                let mut local_env = func.2.clone();
                for (param, arg) in func.0.iter().zip(arguments) {
                    let arg_value = self.eval(arg)?;
                    local_env.insert(param.clone(), arg_value);
                }

                // Temporarily set the environment to the local one
                let old_env = std::mem::replace(&mut self.env, local_env);
                let result = self.eval(&func.1);
                self.env = old_env; // Restore the previous environment

                result
            }
            AST::UnitLiteral => Ok(EvaluationType::Unit),
            AST::NumberLiteral(value) => Ok(EvaluationType::Int(value.parse().unwrap())),
            AST::StringLiteral(value) => Ok(EvaluationType::String(value.clone())),
            AST::BoolLiteral(value) => Ok(EvaluationType::Bool(value.parse().unwrap())),
            AST::Identifier(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            AST::ArrayLiteral(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    values.push(self.eval(elem)?);
                }
                Ok(EvaluationType::Array(values))
            }
            AST::ArrayAccess { array, index } => {
                let array_value = self.eval(array)?;
                let index_value = self.eval(index)?;
                match (array_value, index_value) {
                    (EvaluationType::Array(arr), EvaluationType::Int(idx)) => {
                        if idx < 0 {
                            return Err("Negative array index".to_string());
                        }
                        arr.get(idx as usize)
                            .cloned()
                            .ok_or_else(|| "Index out of bounds".to_string())
                    }
                    _ => Err("Invalid array access".to_string()),
                }
            }
            AST::BinaryExpression {
                left,
                operator,
                right,
            } => {
                let left_value = self.eval(left)?;
                let right_value = self.eval(right)?;

                match (left_value, right_value, operator.as_str()) {
                    // Assignment
                    (lt, rt, "=") => {
                        match &**left {
                            AST::Identifier(name) => {
                                self.env.insert(name.clone(), rt.clone());
                                Ok(rt)
                            }
                            AST::ArrayAccess { array, index } => {
                                let array_name = if let AST::Identifier(name) = &**array {
                                    name
                                } else {
                                    return Err("Left side of assignment must be an identifier or array access".to_string());
                                };

                                let index_value = self.eval(index)?;
                                let index_int = if let EvaluationType::Int(i) = index_value {
                                    i
                                } else {
                                    return Err("Array index must be an integer".to_string());
                                };

                                if index_int < 0 {
                                    return Err("Negative array index".to_string());
                                }

                                let mut array_values = self
                                    .env
                                    .get(array_name)
                                    .ok_or_else(|| format!("Undefined variable: {}", array_name))?
                                    .clone();

                                match &mut array_values {
                                    EvaluationType::Array(arr) => {
                                        let idx = index_int as usize;
                                        if idx >= arr.len() {
                                            // TODO: Resize array if needed??
                                            return Err("Index out of bounds".to_string());
                                        }
                                        arr[idx] = rt.clone();
                                        self.env.insert(array_name.clone(), array_values);
                                        Ok(rt)
                                    }
                                    _ => Err(format!("{array_name} is not an array")),
                                }
                            }
                            _ => Err(
                                "Left side of assignment must be an identifier or array access"
                                    .to_string(),
                            ),
                        }
                    }
                    // Integer arithmetic
                    (EvaluationType::Int(l), EvaluationType::Int(r), "+") => {
                        Ok(EvaluationType::Int(l + r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "-") => {
                        Ok(EvaluationType::Int(l - r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "*") => {
                        Ok(EvaluationType::Int(l * r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "/") => {
                        if r == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(EvaluationType::Int(l / r))
                        }
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "%") => {
                        if r == 0 {
                            Err("Modulo by zero".to_string())
                        } else {
                            Ok(EvaluationType::Int(l % r))
                        }
                    }

                    // Integer comparisons
                    (EvaluationType::Int(l), EvaluationType::Int(r), "==") => {
                        Ok(EvaluationType::Bool(l == r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "!=") => {
                        Ok(EvaluationType::Bool(l != r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "<") => {
                        Ok(EvaluationType::Bool(l < r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "<=") => {
                        Ok(EvaluationType::Bool(l <= r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), ">") => {
                        Ok(EvaluationType::Bool(l > r))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), ">=") => {
                        Ok(EvaluationType::Bool(l >= r))
                    }

                    // Boolean logic
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "&&") => {
                        Ok(EvaluationType::Bool(l && r))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "||") => {
                        Ok(EvaluationType::Bool(l || r))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "==") => {
                        Ok(EvaluationType::Bool(l == r))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "!=") => {
                        Ok(EvaluationType::Bool(l != r))
                    }

                    // String concatenation
                    // (EvaluationType::String(l), EvaluationType::String(r), "+") => {
                    //     Ok(EvaluationType::String(l + &r))
                    // }
                    // (EvaluationType::String(l), EvaluationType::String(r), "==") => {
                    //     Ok(EvaluationType::Bool(l == r))
                    // }
                    // (EvaluationType::String(l), EvaluationType::String(r), "!=") => {
                    //     Ok(EvaluationType::Bool(l != r))
                    // }

                    // Fallback
                    (l, r, op) => Err(format!(
                        "Unsupported binary operation: {:?} {} {:?}",
                        l, op, r
                    )),
                }
            }
            AST::UnaryExpression {
                operator,
                operand,
                is_postfix,
            } => {
                let operand_value = self.eval(operand)?;

                match (operator.as_str(), operand_value.clone(), *is_postfix) {
                    // Prefix integer negation
                    ("-", EvaluationType::Int(value), false) => Ok(EvaluationType::Int(-value)),
                    // Prefix boolean NOT
                    ("!", EvaluationType::Bool(value), false) => Ok(EvaluationType::Bool(!value)),
                    // Prefix increment
                    ("++", EvaluationType::Int(value), false) => Ok(EvaluationType::Int(value + 1)),
                    // Prefix decrement
                    ("--", EvaluationType::Int(value), false) => Ok(EvaluationType::Int(value - 1)),
                    // Postfix increment
                    ("++", EvaluationType::Int(value), true) => Ok(EvaluationType::Int(value)),
                    // Postfix decrement
                    ("--", EvaluationType::Int(value), true) => Ok(EvaluationType::Int(value)),
                    // Add more cases as needed for floats, etc.
                    _ => Err(format!(
                        "Unsupported unary operation: {} (postfix: {}) on {:?}",
                        operator, is_postfix, operand_value
                    )),
                }
            }
            AST::ExpressionStatement(expr) => self.eval(expr),
        }
    }
}
