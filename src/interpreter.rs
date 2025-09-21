use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::error::{ErrorPhase, FlavorError};
use crate::types::{ASTNode as AST, Type};

#[derive(Debug, Clone)]
pub enum EvaluationType {
    Int(i64),
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

    pub fn eval_program(&mut self, nodes: &[AST]) -> Result<EvalOutcome, FlavorError> {
        let mut last_value = EvaluationType::Unit;
        for node in nodes {
            match self.eval(node)? {
                EvalOutcome::Value(val) => last_value = val,
                control_flow => return Ok(control_flow),
            }
        }
        Ok(EvalOutcome::Value(last_value))
    }

    fn collect_array_access_chain<'a>(&self, operand: &'a AST) -> Option<(String, Vec<&'a AST>)> {
        let mut indices = Vec::new();
        let mut current = operand;
        loop {
            match current {
                AST::ArrayAccess { array, index, .. } => {
                    indices.push(index.as_ref());
                    current = array.as_ref();
                }
                AST::Identifier { name, .. } => {
                    indices.reverse();
                    return Some((name.clone(), indices));
                }
                _ => return None,
            }
        }
    }

    fn apply_increment_to_array_access(
        &mut self,
        operand: &AST,
        operator: &str,
        delta: i64,
        is_postfix: bool,
    ) -> Result<EvalOutcome, FlavorError> {
        let (array_name, index_nodes) =
            self.collect_array_access_chain(operand).ok_or_else(|| {
                FlavorError::with_span(
                    "Operand must be an identifier or array access for increment/decrement",
                    operand.span(),
                    ErrorPhase::Runtime,
                )
            })?;

        if index_nodes.is_empty() {
            return Err(FlavorError::with_span(
                "Operand must be an identifier or array access for increment/decrement",
                operand.span(),
                ErrorPhase::Runtime,
            ));
        }

        let mut evaluated_indices = Vec::with_capacity(index_nodes.len());
        for index_expr in &index_nodes {
            let index_value = match self.eval(index_expr)? {
                EvalOutcome::Value(value) => value,
                control_flow => return Ok(control_flow),
            };

            let index_int = if let EvaluationType::Int(i) = index_value {
                i
            } else {
                return Err(FlavorError::with_span(
                    "Array index must be an integer",
                    index_expr.span(),
                    ErrorPhase::Runtime,
                ));
            };

            if index_int < 0 {
                return Err(FlavorError::with_span(
                    "Negative array index",
                    index_expr.span(),
                    ErrorPhase::Runtime,
                ));
            }

            evaluated_indices.push(index_int as usize);
        }

        let Some(root_value) = self.env.get_mut(&array_name) else {
            return Err(FlavorError::with_span(
                format!("Undefined variable: {array_name}"),
                operand.span(),
                ErrorPhase::Runtime,
            ));
        };

        let (old_int, new_int) = Self::mutate_array_element(
            &array_name,
            root_value,
            &evaluated_indices,
            &index_nodes,
            0,
            delta,
            operator,
            is_postfix,
            operand,
        )?;

        let old_eval = EvaluationType::Int(old_int);
        let new_eval = EvaluationType::Int(new_int);

        if is_postfix {
            Ok(EvalOutcome::Value(old_eval))
        } else {
            Ok(EvalOutcome::Value(new_eval))
        }
    }

    fn mutate_array_element(
        array_name: &str,
        current: &mut EvaluationType,
        indices: &[usize],
        index_nodes: &[&AST],
        depth: usize,
        delta: i64,
        operator: &str,
        is_postfix: bool,
        operand: &AST,
    ) -> Result<(i64, i64), FlavorError> {
        let idx = indices[0];
        let index_expr = index_nodes[depth];

        match current {
            EvaluationType::Array(elements) => {
                if idx >= elements.len() {
                    return Err(FlavorError::with_span(
                        "Index out of bounds",
                        index_expr.span(),
                        ErrorPhase::Runtime,
                    ));
                }

                if indices.len() == 1 {
                    match &mut elements[idx] {
                        EvaluationType::Int(element) => {
                            let original = *element;
                            *element += delta;
                            Ok((original, *element))
                        }
                        other => Err(FlavorError::with_span(
                            format!(
                                "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {other:?}"
                            ),
                            operand.span(),
                            ErrorPhase::Runtime,
                        )),
                    }
                } else {
                    Self::mutate_array_element(
                        array_name,
                        &mut elements[idx],
                        &indices[1..],
                        index_nodes,
                        depth + 1,
                        delta,
                        operator,
                        is_postfix,
                        operand,
                    )
                }
            }
            _ => {
                let message = if depth == 0 {
                    format!("{array_name} is not an array")
                } else {
                    "Cannot index into non-array value".to_string()
                };
                let span = if depth == 0 {
                    operand.span()
                } else {
                    index_expr.span()
                };
                Err(FlavorError::with_span(message, span, ErrorPhase::Runtime))
            }
        }
    }

    fn eval(&mut self, node: &AST) -> Result<EvalOutcome, FlavorError> {
        match node {
            AST::Print { expressions, .. } => {
                let mut outputs = Vec::new();
                for expr in expressions {
                    match self.eval(expr)? {
                        EvalOutcome::Value(val) => outputs.push(val.to_string()),
                        control_flow => return Ok(control_flow),
                    }
                }
                println!("{}", outputs.join(""));
                Ok(EvalOutcome::Value(EvaluationType::Unit))
            }
            AST::Body { nodes, .. } => {
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
                ..
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
            AST::While { guard, body, .. } => {
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
                            return Err(FlavorError::with_span(
                                "While guard must evaluate to a boolean",
                                guard.span(),
                                ErrorPhase::Runtime,
                            ));
                        }
                    }
                }
                Ok(EvalOutcome::Value(result))
            }
            AST::LetDeclaration {
                identifier,
                var_type,
                expr,
                span,
            } => {
                let value = match self.eval(expr)? {
                    EvalOutcome::Value(value) => value,
                    control_flow => return Ok(control_flow),
                };
                if let Some(var_type) = var_type {
                    if !value.matches_type(var_type) {
                        return Err(FlavorError::with_span(
                            format!(
                                "Type mismatch: variable '{identifier}' declared as {:?} but value has runtime type {}",
                                var_type,
                                value.type_name()
                            ),
                            *span,
                            ErrorPhase::Runtime,
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
                body,
                ..
            } => {
                let closure_env = Rc::new(RefCell::new(self.env.clone()));
                let func = EvaluationType::Function {
                    parameters: parameters.iter().map(|p| p.0.clone()).collect(),
                    body: Box::new(*body.clone()),
                    env: Rc::clone(&closure_env),
                };
                closure_env.borrow_mut().insert(name.clone(), func.clone());
                self.env.insert(name.clone(), func);
                Ok(EvalOutcome::Value(EvaluationType::Unit))
            }
            AST::Return { expr, .. } => match self.eval(expr)? {
                EvalOutcome::Value(value) => Ok(EvalOutcome::Return(value)),
                control_flow => Ok(control_flow),
            },
            AST::Break { .. } => Ok(EvalOutcome::Break),
            AST::FunctionCall {
                callee,
                arguments,
                span,
            } => {
                let (parameters, body, captured_env, call_span) = match self.eval(callee)? {
                    EvalOutcome::Value(EvaluationType::Function {
                        parameters,
                        body,
                        env,
                    }) => (parameters, body, env, callee.span()),
                    EvalOutcome::Value(_) => {
                        return Err(FlavorError::with_span(
                            "Callee is not a function",
                            callee.span(),
                            ErrorPhase::Runtime,
                        ));
                    }
                    control_flow => return Ok(control_flow),
                };

                if parameters.len() != arguments.len() {
                    return Err(FlavorError::with_span(
                        format!(
                            "Argument count mismatch: expected {}, found {}",
                            parameters.len(),
                            arguments.len()
                        ),
                        *span,
                        ErrorPhase::Runtime,
                    ));
                }

                let mut local_env = captured_env.borrow().clone();
                for (param, arg) in parameters.iter().zip(arguments.iter()) {
                    let arg_value = match self.eval(arg)? {
                        EvalOutcome::Value(value) => value,
                        control_flow => return Ok(control_flow),
                    };
                    local_env.insert(param.clone(), arg_value);
                }

                let old_env = std::mem::replace(&mut self.env, local_env);
                let result = self.eval(&body);
                self.env = old_env;

                match result? {
                    EvalOutcome::Return(value) => Ok(EvalOutcome::Value(value)),
                    EvalOutcome::Value(value) => Ok(EvalOutcome::Value(value)),
                    EvalOutcome::Break => Err(FlavorError::with_span(
                        "Break outside of a loop",
                        call_span,
                        ErrorPhase::Runtime,
                    )),
                }
            }
            AST::FunctionExpression {
                parameters, body, ..
            } => {
                let closure_env = Rc::new(RefCell::new(self.env.clone()));
                let func = EvaluationType::Function {
                    parameters: parameters.iter().map(|p| p.0.clone()).collect(),
                    body: Box::new(*body.clone()),
                    env: Rc::clone(&closure_env),
                };
                Ok(EvalOutcome::Value(func))
            }
            AST::UnitLiteral { .. } => Ok(EvalOutcome::Value(EvaluationType::Unit)),
            AST::NumberLiteral { value, span } => {
                let parsed = value.parse::<i64>().map_err(|_| {
                    FlavorError::with_span("Invalid integer literal", *span, ErrorPhase::Runtime)
                })?;
                Ok(EvalOutcome::Value(EvaluationType::Int(parsed)))
            }
            AST::StringLiteral { value, .. } => {
                Ok(EvalOutcome::Value(EvaluationType::String(value.clone())))
            }
            AST::BoolLiteral { value, span } => {
                let parsed = value.parse::<bool>().map_err(|_| {
                    FlavorError::with_span("Invalid boolean literal", *span, ErrorPhase::Runtime)
                })?;
                Ok(EvalOutcome::Value(EvaluationType::Bool(parsed)))
            }
            AST::Identifier { name, span } => self
                .env
                .get(name)
                .cloned()
                .map(EvalOutcome::Value)
                .ok_or_else(|| {
                    FlavorError::with_span(
                        format!("Undefined variable: {name}"),
                        *span,
                        ErrorPhase::Runtime,
                    )
                }),
            AST::ArrayLiteral { elements, .. } => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    match self.eval(elem)? {
                        EvalOutcome::Value(value) => values.push(value),
                        control_flow => return Ok(control_flow),
                    }
                }
                Ok(EvalOutcome::Value(EvaluationType::Array(values)))
            }
            AST::ArrayAccess { array, index, span } => {
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
                            return Err(FlavorError::with_span(
                                "Negative array index",
                                index.span(),
                                ErrorPhase::Runtime,
                            ));
                        }
                        arr.get(idx as usize)
                            .cloned()
                            .map(EvalOutcome::Value)
                            .ok_or_else(|| {
                                FlavorError::with_span(
                                    "Index out of bounds",
                                    *span,
                                    ErrorPhase::Runtime,
                                )
                            })
                    }
                    _ => Err(FlavorError::with_span(
                        "Invalid array access",
                        *span,
                        ErrorPhase::Runtime,
                    )),
                }
            }
            AST::BinaryExpression {
                left,
                operator,
                right,
                span,
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
                    (_lt, rt, "=") => match &**left {
                        AST::Identifier { name, .. } => {
                            self.env.insert(name.clone(), rt.clone());
                            Ok(EvalOutcome::Value(rt))
                        }
                        AST::ArrayAccess { array, index, .. } => {
                            let array_name = if let AST::Identifier { name, .. } = &**array {
                                name
                            } else {
                                return Err(FlavorError::with_span(
                                    "Left side of assignment must be an identifier or array access",
                                    array.span(),
                                    ErrorPhase::Runtime,
                                ));
                            };

                            let index_value = match self.eval(index)? {
                                EvalOutcome::Value(value) => value,
                                control_flow => return Ok(control_flow),
                            };
                            let index_int = if let EvaluationType::Int(i) = index_value {
                                i
                            } else {
                                return Err(FlavorError::with_span(
                                    "Array index must be an integer",
                                    index.span(),
                                    ErrorPhase::Runtime,
                                ));
                            };

                            if index_int < 0 {
                                return Err(FlavorError::with_span(
                                    "Negative array index",
                                    index.span(),
                                    ErrorPhase::Runtime,
                                ));
                            }

                            let mut array_values =
                                self.env.get(array_name).cloned().ok_or_else(|| {
                                    FlavorError::with_span(
                                        format!("Undefined variable: {array_name}"),
                                        array.span(),
                                        ErrorPhase::Runtime,
                                    )
                                })?;

                            match &mut array_values {
                                EvaluationType::Array(arr) => {
                                    let idx = index_int as usize;
                                    if idx >= arr.len() {
                                        let len = arr.len();
                                        return Err(FlavorError::with_span(
                                            format!(
                                                "Index {idx} out of bounds for array '{array_name}' of length {len}"
                                            ),
                                            index.span(),
                                            ErrorPhase::Runtime,
                                        ));
                                    }
                                    arr[idx] = rt.clone();
                                    self.env.insert(array_name.clone(), array_values);
                                    Ok(EvalOutcome::Value(rt))
                                }
                                _ => Err(FlavorError::with_span(
                                    format!("{array_name} is not an array"),
                                    array.span(),
                                    ErrorPhase::Runtime,
                                )),
                            }
                        }
                        _ => Err(FlavorError::with_span(
                            "Left side of assignment must be an identifier or array access",
                            *span,
                            ErrorPhase::Runtime,
                        )),
                    },
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
                            Err(FlavorError::with_span(
                                "Division by zero",
                                *span,
                                ErrorPhase::Runtime,
                            ))
                        } else {
                            Ok(EvalOutcome::Value(EvaluationType::Int(l / r)))
                        }
                    }
                    (EvaluationType::Int(l), EvaluationType::Int(r), "%") => {
                        if r == 0 {
                            Err(FlavorError::with_span(
                                "Modulo by zero",
                                *span,
                                ErrorPhase::Runtime,
                            ))
                        } else {
                            Ok(EvalOutcome::Value(EvaluationType::Int(l % r)))
                        }
                    }
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
                    (l, r, op) => Err(FlavorError::with_span(
                        format!("Unsupported binary operation: {l:?} {op} {r:?}"),
                        *span,
                        ErrorPhase::Runtime,
                    )),
                }
            }
            AST::UnaryExpression {
                operator,
                operand,
                is_postfix,
                span,
            } => match operator.as_str() {
                "-" if !is_postfix => match self.eval(operand)? {
                    EvalOutcome::Value(EvaluationType::Int(value)) => {
                        Ok(EvalOutcome::Value(EvaluationType::Int(-value)))
                    }
                    EvalOutcome::Value(value) => Err(FlavorError::with_span(
                        format!(
                            "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {value:?}"
                        ),
                        *span,
                        ErrorPhase::Runtime,
                    )),
                    control_flow => Ok(control_flow),
                },
                "!" if !is_postfix => match self.eval(operand)? {
                    EvalOutcome::Value(EvaluationType::Bool(value)) => {
                        Ok(EvalOutcome::Value(EvaluationType::Bool(!value)))
                    }
                    EvalOutcome::Value(value) => Err(FlavorError::with_span(
                        format!(
                            "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {value:?}"
                        ),
                        *span,
                        ErrorPhase::Runtime,
                    )),
                    control_flow => Ok(control_flow),
                },
                "++" | "--" => {
                    let delta: i64 = if operator == "++" { 1 } else { -1 };

                    match &**operand {
                        AST::Identifier { name, .. } => {
                            let current_value = self.env.get(name).cloned().ok_or_else(|| {
                                FlavorError::with_span(
                                    format!("Undefined variable: {name}"),
                                    operand.span(),
                                    ErrorPhase::Runtime,
                                )
                            })?;

                            let old_int = match current_value {
                                EvaluationType::Int(value) => value,
                                other => {
                                    return Err(FlavorError::with_span(
                                        format!(
                                            "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {other:?}"
                                        ),
                                        operand.span(),
                                        ErrorPhase::Runtime,
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
                        AST::ArrayAccess { .. } => self.apply_increment_to_array_access(
                            operand.as_ref(),
                            operator.as_str(),
                            delta,
                            *is_postfix,
                        ),
                        _ => Err(FlavorError::with_span(
                            "Operand must be an identifier or array access for increment/decrement",
                            operand.span(),
                            ErrorPhase::Runtime,
                        )),
                    }
                }
                _ => match self.eval(operand)? {
                    EvalOutcome::Value(operand_value) => Err(FlavorError::with_span(
                        format!(
                            "Unsupported unary operation: {operator} (postfix: {is_postfix}) on {operand_value:?}"
                        ),
                        *span,
                        ErrorPhase::Runtime,
                    )),
                    control_flow => Ok(control_flow),
                },
            },
            AST::ExpressionStatement { expr, .. } => self.eval(expr),
        }
    }
}
