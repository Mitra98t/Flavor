use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::types::{ASTNode as AST, Type};

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
        env: Rc<RefCell<HashMap<String, EvaluationType>>>,
    },
}

impl EvaluationType {
    fn matches_type(&self, expected: &Type) -> bool {
        match (self, expected) {
            (EvaluationType::Int(_), Type::Int) => true,
            (EvaluationType::Bool(_), Type::Bool) => true,
            (EvaluationType::String(_), Type::String) => true,
            (EvaluationType::Unit, Type::Unit) => true,
            (EvaluationType::Array(values), Type::Array(inner)) => {
                values.iter().all(|value| value.matches_type(inner))
            }
            (EvaluationType::Function { .. }, Type::Function { .. }) => true,
            (_, Type::Custom(_)) => true,
            _ => false,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            EvaluationType::Int(_) => "int",
            EvaluationType::Bool(_) => "bool",
            EvaluationType::String(_) => "string",
            EvaluationType::Unit => "unit",
            EvaluationType::Array(_) => "array",
            EvaluationType::Function { .. } => "function",
        }
    }
}

impl fmt::Display for EvaluationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvaluationType::Int(value) => write!(f, "{value}"),
            EvaluationType::Bool(value) => write!(f, "{value}"),
            EvaluationType::String(value) => {
                if let Some(stripped) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
                    write!(f, "{stripped}")
                } else {
                    write!(f, "{value}")
                }
            }
            EvaluationType::Unit => write!(f, "<unit>"),
            EvaluationType::Array(values) => {
                let formatted = values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{formatted}]")
            }
            EvaluationType::Function { .. } => write!(f, "<function>"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalOutcome {
    Value(EvaluationType),
    Break,
    Return(EvaluationType),
}

pub struct Interpreter {
    pub env: HashMap<String, EvaluationType>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: HashMap::new(),
        }
    }

    pub fn eval_program(&mut self, nodes: &[AST]) -> Result<EvalOutcome, String> {
        let mut last_value = EvaluationType::Unit;
        for node in nodes {
            match self.eval(node)? {
                EvalOutcome::Value(val) => last_value = val,
                control_flow => return Ok(control_flow),
            }
        }
        Ok(EvalOutcome::Value(last_value))
    }

    fn eval(&mut self, node: &AST) -> Result<EvalOutcome, String> {
        match node {
            AST::Print(exprs) => {
                let mut outputs = Vec::new();
                for expr in exprs.iter() {
                    match self.eval(expr)? {
                        EvalOutcome::Value(val) => outputs.push(val.to_string()),
                        control_flow => {
                            return Ok(control_flow);
                        }
                    }
                }
                println!("{}", outputs.join(""));
                Ok(EvalOutcome::Value(EvaluationType::Unit))
            }
            AST::Body { nodes } => {
                let mut result = EvaluationType::Unit;
                for n in nodes {
                    match self.eval(n)? {
                        EvalOutcome::Value(val) => result = val,
                        control_flow => return Ok(control_flow),
                    }
                }
                Ok(EvalOutcome::Value(result))
            }
            AST::If {
                guard,
                then_body,
                else_body,
            } => {
                let guard_value = match self.eval(guard)? {
                    EvalOutcome::Value(val) => val,
                    control_flow => return Ok(control_flow),
                };

                if let EvaluationType::Bool(true) = guard_value {
                    self.eval(then_body)
                } else if let Some(else_body) = else_body {
                    self.eval(else_body)
                } else {
                    Ok(EvalOutcome::Value(EvaluationType::Unit))
                }
            }
            AST::While { guard, body } => {
                let mut result = EvaluationType::Unit;
                loop {
                    let guard_value = match self.eval(guard)? {
                        EvalOutcome::Value(val) => val,
                        control_flow => return Ok(control_flow),
                    };
                    match guard_value {
                        EvaluationType::Bool(true) => match self.eval(body)? {
                            EvalOutcome::Value(value) => {
                                result = value;
                            }
                            EvalOutcome::Break => {
                                break;
                            }
                            EvalOutcome::Return(val) => return Ok(EvalOutcome::Return(val)),
                        },
                        EvaluationType::Bool(false) => break,
                        _ => {
                            return Err("While guard must be evaluated to boolean".to_string());
                        }
                    }
                }
                Ok(EvalOutcome::Value(result))
            }
            AST::LetDeclaration {
                identifier,
                var_type,
                expr,
            } => {
                let value = match self.eval(expr)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };
                if let Some(var_type) = var_type {
                    if !value.matches_type(var_type) {
                        return Err(format!(
                            "Type mismatch: variable '{}' declared as {:?} but value has runtime type {}",
                            identifier,
                            var_type,
                            value.type_name()
                        ));
                    }
                }

                if let EvaluationType::Function { env, .. } = &value {
                    env.borrow_mut().insert(identifier.clone(), value.clone());
                }
                self.env.insert(identifier.clone(), value);
                Ok(EvalOutcome::Value(EvaluationType::Unit))
            }
            AST::FunctionDeclaration {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let closure_env = Rc::new(RefCell::new(self.env.clone()));
                let func = EvaluationType::Function {
                    parameters: parameters.clone().iter().map(|p| p.clone().0).collect(),
                    body: Box::new(*body.clone()),
                    env: Rc::clone(&closure_env),
                };
                closure_env.borrow_mut().insert(name.clone(), func.clone());
                self.env.insert(name.clone(), func);
                Ok(EvalOutcome::Value(EvaluationType::Unit))
            }
            AST::Return(expr) => match self.eval(expr)? {
                EvalOutcome::Value(value) => Ok(EvalOutcome::Return(value)),
                contorl_flow => Ok(contorl_flow),
            },
            AST::Break => Ok(EvalOutcome::Break),
            AST::FunctionCall { callee, arguments } => {
                // TODO: rework error handling
                let (parameters, body, captured_env) = match self.eval(callee)? {
                    EvalOutcome::Value(EvaluationType::Function {
                        parameters,
                        body,
                        env,
                    }) => (parameters, body, env),
                    EvalOutcome::Value(_) => return Err("Callee is not a function".to_string()),
                    control_flow => return Ok(control_flow),
                };

                if parameters.len() != arguments.len() {
                    return Err("Argument count mismatch".to_string());
                }

                let mut local_env = captured_env.borrow().clone();
                for (param, arg) in parameters.iter().zip(arguments) {
                    let arg_value = match self.eval(arg)? {
                        EvalOutcome::Value(value) => value,
                        control_flow => return Ok(control_flow),
                    };
                    local_env.insert(param.clone(), arg_value);
                }

                // Temporarily set the environment to the local one
                let old_env = std::mem::replace(&mut self.env, local_env);
                let result = self.eval(&body);
                self.env = old_env; // Restore the previous environment

                match result? {
                    EvalOutcome::Return(value) => Ok(EvalOutcome::Value(value)),
                    EvalOutcome::Value(value) => Ok(EvalOutcome::Value(value)),
                    EvalOutcome::Break => Err("Break outside of a loop".to_string()),
                }
            }
            AST::FunctionExpression {
                parameters,
                return_type: _,
                body,
            } => {
                let closure_env = Rc::new(RefCell::new(self.env.clone()));
                let func = EvaluationType::Function {
                    parameters: parameters.iter().map(|p| p.0.clone()).collect(),
                    body: Box::new(*body.clone()),
                    env: Rc::clone(&closure_env),
                };
                Ok(EvalOutcome::Value(func))
            }
            AST::UnitLiteral => Ok(EvalOutcome::Value(EvaluationType::Unit)),
            AST::NumberLiteral(value) => Ok(EvalOutcome::Value(EvaluationType::Int(
                value.parse().unwrap(),
            ))),
            AST::StringLiteral(value) => {
                Ok(EvalOutcome::Value(EvaluationType::String(value.clone())))
            }
            AST::BoolLiteral(value) => Ok(EvalOutcome::Value(EvaluationType::Bool(
                value.parse().unwrap(),
            ))),
            AST::Identifier(name) => self
                .env
                .get(name)
                .cloned()
                .map(EvalOutcome::Value)
                .ok_or_else(|| format!("Undefined variable: {name}")),
            AST::ArrayLiteral(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    match self.eval(elem)? {
                        EvalOutcome::Value(value) => values.push(value),
                        control_flow => return Ok(control_flow),
                    }
                }
                Ok(EvalOutcome::Value(EvaluationType::Array(values)))
            }
            AST::ArrayAccess { array, index } => {
                let array_value = match self.eval(array)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };
                let index_value = match self.eval(index)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };
                match (array_value, index_value) {
                    (EvaluationType::Array(arr), EvaluationType::Int(idx)) => {
                        if idx < 0 {
                            return Err("Negative array index".to_string());
                        }
                        arr.get(idx as usize)
                            .cloned()
                            .map(EvalOutcome::Value)
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
                let left_value = match self.eval(left)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };
                let right_value = match self.eval(right)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };

                match (left_value, right_value, operator.as_str()) {
                    // Assignment
                    (_lt, rt, "=") => {
                        match &**left {
                            AST::Identifier(name) => {
                                self.env.insert(name.clone(), rt.clone());
                                Ok(EvalOutcome::Value(rt))
                            }
                            AST::ArrayAccess { array, index } => {
                                let array_name = if let AST::Identifier(name) = &**array {
                                    name
                                } else {
                                    return Err("Left side of assignment must be an identifier or array access".to_string());
                                };

                                let index_value = match self.eval(index)? {
                                    EvalOutcome::Value(value) => value,
                                    control_flow => return Ok(control_flow),
                                };
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
                                    .ok_or_else(|| format!("Undefined variable: {array_name}"))?
                                    .clone();

                                match &mut array_values {
                                    EvaluationType::Array(arr) => {
                                        let idx = index_int as usize;
                                        if idx >= arr.len() {
                                            // TODO: For now arrays are fixed-size at runtime; reject implicit resizing.
                                            let len = arr.len();
                                            return Err(format!(
                                                "Index {idx} out of bounds for array '{array_name}' of length {len}"
                                            ));
                                        }
                                        arr[idx] = rt.clone();
                                        self.env.insert(array_name.clone(), array_values);
                                        Ok(EvalOutcome::Value(rt))
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
                        Ok(EvalOutcome::Value(EvaluationType::Int(l + r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "-") => {
                        Ok(EvalOutcome::Value(EvaluationType::Int(l - r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "*") => {
                        Ok(EvalOutcome::Value(EvaluationType::Int(l * r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "/") => {
                        if r == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(EvalOutcome::Value(EvaluationType::Int(l / r)))
                        }
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "%") => {
                        if r == 0 {
                            Err("Modulo by zero".to_string())
                        } else {
                            Ok(EvalOutcome::Value(EvaluationType::Int(l % r)))
                        }
                    }

                    // Integer comparisons
                    (EvaluationType::Int(l), EvaluationType::Int(r), "==") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l == r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "!=") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l != r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "<") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l < r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "<=") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l <= r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), ">") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l > r)))
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), ">=") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l >= r)))
                    }

                    // Boolean logic
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "&&") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l && r)))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "||") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l || r)))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "==") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l == r)))
                    }
                    (EvaluationType::Bool(l), EvaluationType::Bool(r), "!=") => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(l != r)))
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
                    (l, r, op) => Err(format!("Unsupported binary operation: {l:?} {op} {r:?}",)),
                }
            }
            AST::UnaryExpression {
                operator,
                operand,
                is_postfix,
            } => match operator.as_str() {
                "-" if !is_postfix => match self.eval(operand)? {
                    EvalOutcome::Value(EvaluationType::Int(value)) => {
                        Ok(EvalOutcome::Value(EvaluationType::Int(-value)))
                    }
                    EvalOutcome::Value(value) => Err(format!(
                        "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {value:?}",
                    )),
                    control_flow => Ok(control_flow),
                },
                "!" if !is_postfix => match self.eval(operand)? {
                    EvalOutcome::Value(EvaluationType::Bool(value)) => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(!value)))
                    }
                    EvalOutcome::Value(value) => Err(format!(
                        "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {value:?}",
                    )),
                    control_flow => Ok(control_flow),
                },
                "++" | "--" => {
                    let delta: i64 = if operator == "++" { 1 } else { -1 };

                    match &**operand {
                        AST::Identifier(name) => {
                            let current_value = self
                                .env
                                .get(name)
                                .cloned()
                                .ok_or_else(|| format!("Undefined variable: {name}"))?;

                            let old_int = match current_value {
                                EvaluationType::Int(value) => value,
                                other => {
                                    return Err(format!(
                                        "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {other:?}",
                                    ));
                                }
                            };

                            let new_int = old_int + delta;
                            let new_value = EvaluationType::Int(new_int);
                            let old_value = EvaluationType::Int(old_int);

                            self.env.insert(name.clone(), new_value.clone());

                            if *is_postfix {
                                Ok(EvalOutcome::Value(old_value))
                            } else {
                                Ok(EvalOutcome::Value(new_value))
                            }
                        }
                        AST::ArrayAccess { array, index } => {
                            let array_name = if let AST::Identifier(name) = &**array {
                                name
                            } else {
                                return Err(
                                    "Left side of increment must be an identifier or array access"
                                        .to_string(),
                                );
                            };

                            let index_value = match self.eval(index)? {
                                EvalOutcome::Value(value) => value,
                                control_flow => return Ok(control_flow),
                            };
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
                                .ok_or_else(|| format!("Undefined variable: {array_name}"))?
                                .clone();

                            let (old_int, new_int) = match &mut array_values {
                                EvaluationType::Array(arr) => {
                                    let idx = index_int as usize;
                                    if idx >= arr.len() {
                                        return Err("Index out of bounds".to_string());
                                    }

                                    match arr.get_mut(idx) {
                                        Some(EvaluationType::Int(element)) => {
                                            let old_value = *element;
                                            *element += delta;
                                            (old_value, *element)
                                        }
                                        Some(other) => {
                                            return Err(format!(
                                                "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {other:?}",
                                            ));
                                        }
                                        None => unreachable!(),
                                    }
                                }
                                _ => return Err(format!("{array_name} is not an array")),
                            };

                            self.env.insert(array_name.clone(), array_values);

                            let old_value = EvaluationType::Int(old_int);
                            let new_value = EvaluationType::Int(new_int);

                            if *is_postfix {
                                Ok(EvalOutcome::Value(old_value))
                            } else {
                                Ok(EvalOutcome::Value(new_value))
                            }
                        }
                        _ => Err(
                            "Operand must be an identifier or array access for increment/decrement"
                                .to_string(),
                        ),
                    }
                }
                _ => match self.eval(operand)? {
                    EvalOutcome::Value(operand_value) => Err(format!(
                        "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {operand_value:?}",
                    )),
                    control_flow => Ok(control_flow),
                },
            },
            AST::ExpressionStatement(expr) => self.eval(expr),
        }
    }
}
