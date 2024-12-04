use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::ast::{Expr, Stmt, BinaryOperator};
use crate::types::{Context, Confidence};
use crate::context::ContextManager;
use crate::stdlib::StandardLibrary;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct Function {
    parameters: Vec<String>,
    body: Vec<Stmt>,
    confidence_level: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Float(f64),
    Integer(i64),
    String(String),
    Confidence(Confidence),
    Context(Context),
    Array(Vec<Value>),
    Function(Function),
    Return(Box<Value>),
    Error(RuntimeError),
    Struct(StructInstance),
    Promise(Arc<Mutex<PromiseState>>),
}

#[derive(Debug, Clone)]
pub enum PromiseState {
    Pending(Option<f64>),  // Optional confidence for pending state
    Resolved(Value),
    Rejected(RuntimeError),
}

impl Value {
    pub fn with_confidence(&self, confidence: f64) -> Self {
        match self {
            Value::Float(f) => Value::Float(f * confidence),
            Value::Integer(i) => Value::Integer(i as i64 * confidence as i64),
            Value::Confidence(conf) => Value::Confidence(Confidence::new(conf.value * confidence)?),
            Value::Context(ctx) => Value::Context(Context::new(ctx.name, ctx.value, ctx.confidence, ctx.bounds)?),
            Value::Array(arr) => Value::Array(arr.iter().map(|v| v.with_confidence(confidence)).collect()),
            Value::Function(func) => Value::Function(Function {
                parameters: func.parameters.clone(),
                body: func.body.clone(),
                confidence_level: func.confidence_level.clone(),
            }),
            Value::Return(ret) => Value::Return(Box::new(ret.with_confidence(confidence))),
            Value::Error(error) => Value::Error(error.with_confidence(confidence)),
            Value::Struct(instance) => Value::Struct(StructInstance {
                struct_type: instance.struct_type,
                confidence: instance.confidence,
                fields: instance.fields.iter().map(|(k, v)| (k.clone(), v.with_confidence(confidence))).collect(),
            }),
            Value::Promise(promise) => Value::Promise(promise.clone()),
        }
    }
    
    pub fn confidence(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(f),
            Value::Integer(i) => Some(i as f64),
            Value::Confidence(conf) => Some(conf.value),
            Value::Context(ctx) => Some(ctx.confidence),
            Value::Array(arr) => {
                let mut confidence = None;
                for v in arr {
                    if let Some(c) = v.confidence() {
                        if let Some(existing) = confidence {
                            confidence = Some(existing.min(c));
                        } else {
                            confidence = Some(c);
                        }
                    }
                }
                confidence
            },
            Value::Function(func) => func.confidence_level,
            Value::Return(ret) => ret.confidence(),
            Value::Error(error) => error.confidence(),
            Value::Struct(instance) => instance.confidence,
            Value::Promise(promise) => {
                let state = promise.lock().unwrap();
                match &*state {
                    PromiseState::Pending(conf) => conf,
                    PromiseState::Resolved(value) => value.confidence(),
                    PromiseState::Rejected(error) => error.confidence(),
                }
            },
        }
    }
    
    pub fn as_promise(&self) -> Result<Arc<Mutex<PromiseState>>, RuntimeError> {
        match self {
            Value::Promise(promise) => Ok(promise.clone()),
            _ => Err(RuntimeError::Error {
                message: "Value is not a promise".to_string(),
                code: Some("TYPE_ERROR".to_string()),
                confidence: Some(1.0),
                context: None,
            }),
        }
    }
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    context_manager: ContextManager,
    stdlib: StandardLibrary,
    return_value: Option<Value>,
    module_loader: ModuleLoader,
    struct_definitions: HashMap<String, StructDefinition>,
    trait_definitions: HashMap<String, TraitDefinition>,
    trait_implementations: HashMap<String, Vec<TraitImplementation>>,
    type_env: TypeEnvironment,
    generic_struct_definitions: HashMap<String, GenericStructDefinition>,
    generic_trait_definitions: HashMap<String, GenericTraitDefinition>,
    async_functions: HashMap<String, AsyncFunction>,
    operator_definitions: HashMap<(Type, Operator, Option<Type>), OperatorDefinition>,
}

#[derive(Debug)]
pub enum ControlFlow {
    Break,
    Continue,
    Return(Value),
    None,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Error {
        message: String,
        code: Option<String>,
        confidence: Option<f64>,
        context: Option<String>,
    },
    Throw(Value),
}

impl RuntimeError {
    pub fn with_confidence(&self, confidence: f64) -> Self {
        match self {
            RuntimeError::Error { message, code, confidence: _, context } => {
                RuntimeError::Error {
                    message: message.clone(),
                    code: code.clone(),
                    confidence: Some(confidence),
                    context: context.clone(),
                }
            },
            RuntimeError::Throw(value) => {
                RuntimeError::Throw(value.with_confidence(confidence))
            },
        }
    }
    
    pub fn confidence(&self) -> Option<f64> {
        match self {
            RuntimeError::Error { confidence, .. } => *confidence,
            RuntimeError::Throw(value) => value.confidence(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub name: String,
    pub confidence: Option<f64>,
    pub exports: HashMap<String, Value>,
    pub path: PathBuf,
}

pub struct ModuleLoader {
    modules: HashMap<String, ModuleInstance>,
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            search_paths: vec![PathBuf::from(".")],
        }
    }
    
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    pub fn load_module(&mut self, name: &str, interpreter: &mut Interpreter) -> Result<ModuleInstance, RuntimeError> {
        // Check if module is already loaded
        if let Some(module) = self.modules.get(name) {
            return Ok(module.clone());
        }
        
        // Find module file
        let module_path = self.find_module(name)?;
        
        // Parse and execute module
        let source = fs::read_to_string(&module_path)
            .map_err(|e| RuntimeError::Error {
                message: format!("Failed to read module '{}': {}", name, e),
                code: Some("MODULE_READ_ERROR".to_string()),
                confidence: Some(1.0),
                context: None,
            })?;
        
        // Create new module instance
        let mut module = ModuleInstance {
            name: name.to_string(),
            confidence: None,
            exports: HashMap::new(),
            path: module_path,
        };
        
        // Create new scope for module
        let old_scope = interpreter.variables.clone();
        interpreter.variables.clear();
        
        // Execute module code
        let ast = interpreter.parse(&source)?;
        interpreter.execute_module(&ast, &mut module)?;
        
        // Restore original scope
        interpreter.variables = old_scope;
        
        // Cache module
        self.modules.insert(name.to_string(), module.clone());
        
        Ok(module)
    }
    
    fn find_module(&self, name: &str) -> Result<PathBuf, RuntimeError> {
        for path in &self.search_paths {
            let module_path = path.join(format!("{}.prism", name));
            if module_path.exists() {
                return Ok(module_path);
            }
        }
        
        Err(RuntimeError::Error {
            message: format!("Module '{}' not found", name),
            code: Some("MODULE_NOT_FOUND".to_string()),
            confidence: Some(1.0),
            context: None,
        })
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            context_manager: ContextManager::new(0.5),
            stdlib: StandardLibrary::new(),
            return_value: None,
            module_loader: ModuleLoader::new(),
            struct_definitions: HashMap::new(),
            trait_definitions: HashMap::new(),
            trait_implementations: HashMap::new(),
            type_env: TypeEnvironment::new(),
            generic_struct_definitions: HashMap::new(),
            generic_trait_definitions: HashMap::new(),
            async_functions: HashMap::new(),
            operator_definitions: HashMap::new(),
        }
    }

    pub fn eval(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match self.eval_with_control(stmt)? {
            (value, ControlFlow::Return(ret)) => Ok(ret),
            (value, _) => Ok(value),
        }
    }

    fn eval_with_control(&mut self, stmt: &Stmt) -> Result<(Value, ControlFlow), String> {
        match stmt {
            Stmt::ForLoop { initializer, condition, increment, body } => {
                // Create new scope for loop variables
                let old_scope = self.variables.clone();
                
                // Initialize
                if let Some(init) = initializer {
                    self.eval(init)?;
                }
                
                let mut last_value = Value::Void;
                
                loop {
                    // Check condition
                    if let Some(cond) = condition {
                        match self.eval_expr(cond)? {
                            Value::Float(f) if f > 0.0 => {},
                            Value::Integer(i) if i > 0 => {},
                            _ => break,
                        }
                    }
                    
                    // Execute body
                    for stmt in body {
                        match self.eval_with_control(stmt)? {
                            (value, ControlFlow::Break) => {
                                self.variables = old_scope;
                                return Ok((value, ControlFlow::None));
                            },
                            (value, ControlFlow::Continue) => break,
                            (value, ControlFlow::Return(ret)) => {
                                self.variables = old_scope;
                                return Ok((value, ControlFlow::Return(ret)));
                            },
                            (value, ControlFlow::None) => {
                                last_value = value;
                            },
                        }
                    }
                    
                    // Increment
                    if let Some(inc) = increment {
                        self.eval_expr(inc)?;
                    }
                }
                
                self.variables = old_scope;
                Ok((last_value, ControlFlow::None))
            },
            
            Stmt::ForInLoop { variable, iterator, body } => {
                let old_scope = self.variables.clone();
                let mut last_value = Value::Void;
                
                // Evaluate iterator
                let iter_value = self.eval_expr(iterator)?;
                let values = match iter_value {
                    Value::Array(arr) => arr,
                    Value::Range { start, end, step } => {
                        let mut values = Vec::new();
                        let mut current = start;
                        while current < end {
                            values.push(Value::Float(current));
                            current += step.unwrap_or(1.0);
                        }
                        values
                    },
                    _ => return Err("Can only iterate over arrays and ranges".to_string()),
                };
                
                // Iterate over values
                for value in values {
                    self.variables.insert(variable.clone(), value);
                    
                    for stmt in body {
                        match self.eval_with_control(stmt)? {
                            (value, ControlFlow::Break) => {
                                self.variables = old_scope;
                                return Ok((value, ControlFlow::None));
                            },
                            (value, ControlFlow::Continue) => break,
                            (value, ControlFlow::Return(ret)) => {
                                self.variables = old_scope;
                                return Ok((value, ControlFlow::Return(ret)));
                            },
                            (value, ControlFlow::None) => {
                                last_value = value;
                            },
                        }
                    }
                }
                
                self.variables = old_scope;
                Ok((last_value, ControlFlow::None))
            },
            
            Stmt::WhileLoop { condition, body } => {
                let mut last_value = Value::Void;
                
                loop {
                    // Check condition
                    match self.eval_expr(condition)? {
                        Value::Float(f) if f > 0.0 => {},
                        Value::Integer(i) if i > 0 => {},
                        _ => break,
                    }
                    
                    // Execute body
                    for stmt in body {
                        match self.eval_with_control(stmt)? {
                            (value, ControlFlow::Break) => {
                                return Ok((value, ControlFlow::None));
                            },
                            (value, ControlFlow::Continue) => break,
                            (value, ControlFlow::Return(ret)) => {
                                return Ok((value, ControlFlow::Return(ret)));
                            },
                            (value, ControlFlow::None) => {
                                last_value = value;
                            },
                        }
                    }
                }
                
                Ok((last_value, ControlFlow::None))
            },
            
            Stmt::Break => Ok((Value::Void, ControlFlow::Break)),
            
            Stmt::Continue => Ok((Value::Void, ControlFlow::Continue)),
            
            // Handle other statements
            _ => {
                let value = self.eval(stmt)?;
                Ok((value, ControlFlow::None))
            },
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Integer(i) => Ok(Value::Integer(*i)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable '{}' not found", name))
            },
            
            Expr::ConfidenceValue { value } => {
                let val = self.eval_expr(value)?;
                match val {
                    Value::Float(f) => Ok(Value::Confidence(Confidence::new(f)?)),
                    _ => Err("Confidence value must be a float".to_string()),
                }
            },
            
            Expr::ConfidenceFlow { source, target } => {
                let source_val = self.eval_expr(source)?;
                let target_val = self.eval_expr(target)?;
                match (source_val, target_val) {
                    (Value::Float(s), Value::Float(t)) => Ok(Value::Float(s * t)),
                    _ => Err("Confidence flow requires float values".to_string()),
                }
            },
            
            Expr::ReverseConfidenceFlow { source, target } => {
                let evaluated = self.eval_expr(source)?;
                self.variables.insert(target.to_string(), evaluated.clone());
                Ok(evaluated)
            },
            
            Expr::UncertainIf { condition, high_confidence, medium_confidence, low_confidence } => {
                let cond_val = self.eval_expr(condition)?;
                match cond_val {
                    Value::Float(conf) => {
                        let mut result = Value::Void;
                        if conf >= 0.7 {
                            for stmt in high_confidence {
                                result = self.eval(stmt)?;
                            }
                        } else if conf >= 0.4 {
                            if let Some(stmts) = medium_confidence {
                                for stmt in stmts {
                                    result = self.eval(stmt)?;
                                }
                            }
                        } else if let Some(stmts) = low_confidence {
                            for stmt in stmts {
                                result = self.eval(stmt)?;
                            }
                        }
                        Ok(result)
                    },
                    _ => Err("Condition must evaluate to a confidence value".to_string())
                }
            },
            
            Expr::ContextBlock { context_name, body } => {
                self.context_manager.enter_context(context_name.clone(), 0.5)?;
                let mut result = Value::Void;
                for stmt in body {
                    result = self.eval(stmt)?;
                }
                self.context_manager.exit_context()?;
                Ok(result)
            },
            
            Expr::ContextShift { from, to, body } => {
                let mut results = Vec::new();
                for stmt in body {
                    results.push(self.eval(stmt)?);
                }
                Ok(Value::Context(Context::new(
                    from.clone(),
                    vec![],
                    0.5,
                    vec![to.clone()]
                )?))
            },
            
            Expr::BinaryOp { op, left, right } => {
                let lhs = self.eval_expr(left)?;
                let rhs = self.eval_expr(right)?;
                
                // Try to find custom operator implementation
                if let Some(def) = self.find_operator_definition(op, &lhs, &rhs) {
                    self.eval_custom_operator(def, lhs, rhs)
                } else {
                    // Fall back to built-in operator implementation
                    self.eval_builtin_operator(op, lhs, rhs)
                }
            },
            
            Expr::UnaryOp { op, expr } => {
                let value = self.eval_expr(expr)?;
                
                // Try to find custom operator implementation
                if let Some(def) = self.find_unary_operator_definition(op, &value) {
                    self.eval_custom_unary_operator(def, value)
                } else {
                    // Fall back to built-in operator implementation
                    self.eval_builtin_unary_operator(op, value)
                }
            },
            
            Expr::Verify { sources: _, threshold: _, body } => {
                // Placeholder for verification logic
                let mut result = Value::Void;
                for stmt in body {
                    result = self.eval(stmt)?;
                }
                Ok(result)
            },
            
            Expr::FunctionCall { name, arguments } => {
                let function = self.functions.get(name)
                    .ok_or_else(|| format!("Function '{}' not found", name))?
                    .clone();
                
                if arguments.len() != function.parameters.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments but got {}",
                        name,
                        function.parameters.len(),
                        arguments.len()
                    ));
                }
                
                // Create new scope for function execution
                let mut function_scope = HashMap::new();
                
                // Evaluate and bind arguments to parameters
                for (param, arg) in function.parameters.iter().zip(arguments.iter()) {
                    let value = self.eval_expr(arg)?;
                    function_scope.insert(param.clone(), value);
                }
                
                // Save current scope and set function scope
                let old_scope = std::mem::replace(&mut self.variables, function_scope);
                self.return_value = None;
                
                // Execute function body
                let mut result = Value::Void;
                for stmt in &function.body {
                    result = self.eval(stmt)?;
                    if let Some(return_value) = self.return_value.take() {
                        // Restore original scope
                        self.variables = old_scope;
                        
                        // Apply confidence level if specified
                        if let Some(conf) = function.confidence_level {
                            match return_value {
                                Value::Float(f) => return Ok(Value::Float(f * conf)),
                                Value::Integer(i) => return Ok(Value::Float(i as f64 * conf)),
                                _ => return Ok(return_value),
                            }
                        }
                        return Ok(return_value);
                    }
                }
                
                // Restore original scope
                self.variables = old_scope;
                
                // Apply confidence level to final result if no explicit return
                if let Some(conf) = function.confidence_level {
                    match result {
                        Value::Float(f) => Ok(Value::Float(f * conf)),
                        Value::Integer(i) => Ok(Value::Float(i as f64 * conf)),
                        _ => Ok(result),
                    }
                } else {
                    Ok(result)
                }
            },
            
            Expr::Return(value) => {
                match value {
                    Some(expr) => {
                        let val = self.eval_expr(expr)?;
                        self.return_value = Some(val.clone());
                        Ok(val)
                    },
                    None => {
                        self.return_value = Some(Value::Void);
                        Ok(Value::Void)
                    }
                }
            },
            
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::Array(values))
            },
            
            Expr::ArrayAccess { array, index } => {
                let array_val = self.eval_expr(array)?;
                let index_val = self.eval_expr(index)?;
                
                match (array_val, index_val) {
                    (Value::Array(arr), Value::Integer(i)) => {
                        if i < 0 || i >= arr.len() as i64 {
                            Err(format!("Array index out of bounds: {}", i))
                        } else {
                            Ok(arr[i as usize].clone())
                        }
                    },
                    _ => Err("Array access requires an array and integer index".to_string()),
                }
            },
            
            Expr::ArraySlice { array, start, end } => {
                let array_val = self.eval_expr(array)?;
                
                match array_val {
                    Value::Array(arr) => {
                        let start_idx = match start {
                            Some(expr) => match self.eval_expr(expr)? {
                                Value::Integer(i) => i.max(0) as usize,
                                _ => return Err("Slice start must be an integer".to_string()),
                            },
                            None => 0,
                        };
                        
                        let end_idx = match end {
                            Some(expr) => match self.eval_expr(expr)? {
                                Value::Integer(i) => i.min(arr.len() as i64) as usize,
                                _ => return Err("Slice end must be an integer".to_string()),
                            },
                            None => arr.len(),
                        };
                        
                        if start_idx <= end_idx {
                            Ok(Value::Array(arr[start_idx..end_idx].to_vec()))
                        } else {
                            Ok(Value::Array(vec![]))
                        }
                    },
                    _ => Err("Can only slice arrays".to_string()),
                }
            },
            
            Expr::ArrayLength(array) => {
                let array_val = self.eval_expr(array)?;
                match array_val {
                    Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                    _ => Err("Length operation requires an array".to_string()),
                }
            },
            
            Expr::ArrayPush { array, value } => {
                let array_val = self.eval_expr(array)?;
                let push_val = self.eval_expr(value)?;
                
                match array_val {
                    Value::Array(mut arr) => {
                        arr.push(push_val);
                        Ok(Value::Array(arr))
                    },
                    _ => Err("Push operation requires an array".to_string()),
                }
            },
            
            Expr::ArrayPop(array) => {
                let array_val = self.eval_expr(array)?;
                match array_val {
                    Value::Array(mut arr) => {
                        arr.pop().ok_or_else(|| "Cannot pop from empty array".to_string())
                    },
                    _ => Err("Pop operation requires an array".to_string()),
                }
            },
            
            Expr::Range { start, end, step } => {
                let start_val = self.eval_expr(start)?;
                let end_val = self.eval_expr(end)?;
                let step_val = if let Some(step_expr) = step {
                    Some(self.eval_expr(step_expr)?)
                } else {
                    None
                };
                
                match (start_val, end_val, step_val) {
                    (Value::Float(s), Value::Float(e), Some(Value::Float(step))) => {
                        Ok(Value::Range { start: s, end: e, step: Some(step) })
                    },
                    (Value::Float(s), Value::Float(e), None) => {
                        Ok(Value::Range { start: s, end: e, step: Some(1.0) })
                    },
                    (Value::Integer(s), Value::Integer(e), Some(Value::Integer(step))) => {
                        Ok(Value::Range {
                            start: s as f64,
                            end: e as f64,
                            step: Some(step as f64),
                        })
                    },
                    (Value::Integer(s), Value::Integer(e), None) => {
                        Ok(Value::Range {
                            start: s as f64,
                            end: e as f64,
                            step: Some(1.0),
                        })
                    },
                    _ => Err("Range requires numeric values".to_string()),
                }
            },
            
            Expr::Match { value, cases } => {
                let match_value = self.eval_expr(value)?;
                
                // Create new scope for pattern bindings
                let old_scope = self.variables.clone();
                
                for case in cases {
                    // Try to match the pattern
                    if let Some(bindings) = self.match_pattern(&match_value, &case.pattern)? {
                        // Apply bindings to scope
                        for (name, value) in bindings {
                            self.variables.insert(name, value);
                        }
                        
                        // Check guard if present
                        if let Some(guard) = &case.guard {
                            match self.eval_expr(guard)? {
                                Value::Float(f) if f > 0.0 => {},
                                Value::Integer(i) if i > 0 => {},
                                _ => {
                                    self.variables = old_scope.clone();
                                    continue;
                                }
                            }
                        }
                        
                        // Execute case body
                        let mut result = Value::Void;
                        for stmt in &case.body {
                            result = self.eval(stmt)?;
                        }
                        
                        // Restore original scope
                        self.variables = old_scope;
                        return Ok(result);
                    }
                }
                
                // No pattern matched
                Err("No pattern matched the value".to_string())
            },
            
            Expr::Try(expr) => {
                match self.eval_expr(expr) {
                    Ok(value) => Ok(value),
                    Err(error) => Ok(Value::Error(error)),
                }
            },
            
            Expr::Throw { error, confidence } => {
                let error_value = self.eval_expr(error)?;
                let error = RuntimeError::Throw(error_value);
                
                if let Some(conf_expr) = confidence {
                    let conf_value = self.eval_expr(conf_expr)?;
                    match conf_value {
                        Value::Float(conf) => Err(error.with_confidence(conf)),
                        _ => Err(error),
                    }
                } else {
                    Err(error)
                }
            },
            
            Expr::Error { message, code, confidence, context } => {
                Err(RuntimeError::Error {
                    message,
                    code,
                    confidence,
                    context,
                })
            },
            
            Expr::StructInstantiation { name, fields } => {
                let struct_def = self.struct_definitions.get(name)
                    .ok_or_else(|| RuntimeError::Error {
                        message: format!("Struct '{}' not found", name),
                        code: Some("TYPE_ERROR".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    })?;
                
                let mut instance_fields = HashMap::new();
                
                // Initialize fields with default values
                for (field_name, field_def) in &struct_def.fields {
                    if let Some(default) = &field_def.default_value {
                        let value = if let Some(conf) = field_def.confidence {
                            default.with_confidence(conf)
                        } else {
                            default.clone()
                        };
                        instance_fields.insert(field_name.clone(), value);
                    }
                }
                
                // Set provided field values
                for (field_name, value_expr) in fields {
                    let value = self.eval_expr(value_expr)?;
                    
                    if let Some(field_def) = struct_def.fields.get(field_name) {
                        let value = if let Some(conf) = field_def.confidence {
                            value.with_confidence(conf)
                        } else {
                            value
                        };
                        instance_fields.insert(field_name.clone(), value);
                    } else {
                        return Err(RuntimeError::Error {
                            message: format!("Field '{}' not found in struct '{}'", field_name, name),
                            code: Some("FIELD_NOT_FOUND".to_string()),
                            confidence: Some(1.0),
                            context: None,
                        });
                    }
                }
                
                Ok(Value::Struct(StructInstance {
                    struct_type: name.clone(),
                    confidence: struct_def.confidence,
                    fields: instance_fields,
                }))
            },
            
            Expr::FieldAccess { object, field } => {
                let obj_value = self.eval_expr(object)?;
                obj_value.get_field(field)
            },
            
            Expr::MethodCall { object, method, arguments } => {
                let mut obj_value = self.eval_expr(object)?;
                
                match &obj_value {
                    Value::Struct(instance) => {
                        let struct_def = self.struct_definitions.get(&instance.struct_type)
                            .ok_or_else(|| RuntimeError::Error {
                                message: format!("Struct '{}' not found", instance.struct_type),
                                code: Some("TYPE_ERROR".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            })?;
                        
                        let method_def = struct_def.methods.get(method)
                            .ok_or_else(|| RuntimeError::Error {
                                message: format!("Method '{}' not found", method),
                                code: Some("METHOD_NOT_FOUND".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            })?;
                        
                        if arguments.len() != method_def.parameters.len() {
                            return Err(RuntimeError::Error {
                                message: format!(
                                    "Method '{}' expects {} arguments but got {}",
                                    method,
                                    method_def.parameters.len(),
                                    arguments.len()
                                ),
                                code: Some("ARGUMENT_ERROR".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            });
                        }
                        
                        // Create new scope for method execution
                        let old_scope = self.variables.clone();
                        self.variables.clear();
                        
                        // Bind 'this' to the current instance
                        self.variables.insert("this".to_string(), obj_value.clone());
                        
                        // Bind arguments to parameters
                        for (param, arg) in method_def.parameters.iter().zip(arguments) {
                            let value = self.eval_expr(arg)?;
                            self.variables.insert(param.clone(), value);
                        }
                        
                        // Execute method body
                        let mut result = Value::Void;
                        for stmt in &method_def.body {
                            result = self.eval_stmt(stmt)?;
                        }
                        
                        // Apply method confidence if specified
                        if let Some(conf) = method_def.confidence {
                            result = result.with_confidence(conf);
                        }
                        
                        // Restore original scope
                        self.variables = old_scope;
                        
                        Ok(result)
                    },
                    _ => Err(RuntimeError::Error {
                        message: "Not a struct".to_string(),
                        code: Some("TYPE_ERROR".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    }),
                }
            },
            
            Expr::This => {
                self.variables.get("this")
                    .cloned()
                    .ok_or_else(|| RuntimeError::Error {
                        message: "'this' can only be used inside methods".to_string(),
                        code: Some("CONTEXT_ERROR".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    })
            },
            
            Expr::DynamicMethodCall { object, trait_name, method, arguments } => {
                let obj_value = self.eval_expr(object)?;
                
                match &obj_value {
                    Value::Struct(instance) => {
                        // Find trait implementation
                        let impls = self.trait_implementations.get(&instance.struct_type)
                            .ok_or_else(|| RuntimeError::Error {
                                message: format!("No traits implemented for struct '{}'", instance.struct_type),
                                code: Some("NO_TRAIT_IMPL".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            })?;
                        
                        let impl_def = impls.iter()
                            .find(|imp| imp.trait_name == *trait_name)
                            .ok_or_else(|| RuntimeError::Error {
                                message: format!("Trait '{}' not implemented for struct '{}'", trait_name, instance.struct_type),
                                code: Some("TRAIT_NOT_IMPLEMENTED".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            })?;
                        
                        let method_def = impl_def.methods.get(method)
                            .ok_or_else(|| RuntimeError::Error {
                                message: format!("Method '{}' not found in trait implementation", method),
                                code: Some("METHOD_NOT_FOUND".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            })?;
                        
                        if arguments.len() != method_def.parameters.len() {
                            return Err(RuntimeError::Error {
                                message: format!(
                                    "Method '{}' expects {} arguments but got {}",
                                    method,
                                    method_def.parameters.len(),
                                    arguments.len()
                                ),
                                code: Some("ARGUMENT_ERROR".to_string()),
                                confidence: Some(1.0),
                                context: None,
                            });
                        }
                        
                        // Create new scope for method execution
                        let old_scope = self.variables.clone();
                        self.variables.clear();
                        
                        // Bind 'this' to the current instance
                        self.variables.insert("this".to_string(), obj_value.clone());
                        
                        // Bind arguments to parameters
                        for (param, arg) in method_def.parameters.iter().zip(arguments) {
                            let value = self.eval_expr(arg)?;
                            self.variables.insert(param.clone(), value);
                        }
                        
                        // Execute method body
                        let mut result = Value::Void;
                        for stmt in &method_def.body {
                            result = self.eval_stmt(stmt)?;
                        }
                        
                        // Apply confidence adjustments
                        let trait_conf = impl_def.confidence.unwrap_or(1.0);
                        let method_conf = method_def.confidence.unwrap_or(1.0);
                        result = result.with_confidence(trait_conf * method_conf);
                        
                        // Restore original scope
                        self.variables = old_scope;
                        
                        Ok(result)
                    },
                    _ => Err(RuntimeError::Error {
                        message: "Not a struct".to_string(),
                        code: Some("TYPE_ERROR".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    }),
                }
            },
        }
    }
    
    fn match_pattern(&self, value: &Value, pattern: &Pattern) -> Result<Option<HashMap<String, Value>>, String> {
        let mut bindings = HashMap::new();
        
        match pattern {
            Pattern::Literal(pat_val) => {
                if value == pat_val {
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            },
            
            Pattern::Variable(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(Some(bindings))
            },
            
            Pattern::Array(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(None);
                    }
                    
                    for (pat, val) in patterns.iter().zip(values) {
                        if let Some(mut pat_bindings) = self.match_pattern(val, pat)? {
                            bindings.extend(pat_bindings);
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            },
            
            Pattern::Rest(rest_pattern) => {
                if let Value::Array(values) = value {
                    for val in values {
                        if let Some(mut pat_bindings) = self.match_pattern(val, rest_pattern)? {
                            bindings.extend(pat_bindings);
                        }
                    }
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            },
            
            Pattern::Confidence { value: pat_value, range } => {
                match value {
                    Value::Float(f) => {
                        if *f >= range.0 && *f <= range.1 {
                            self.match_pattern(value, pat_value)
                        } else {
                            Ok(None)
                        }
                    },
                    _ => Ok(None),
                }
            },
            
            Pattern::Context { name, pattern } => {
                if let Value::Context(ctx) = value {
                    if ctx.name() == name {
                        self.match_pattern(&ctx.value(), pattern)
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            },
            
            Pattern::Or(patterns) => {
                for pat in patterns {
                    if let Some(pat_bindings) = self.match_pattern(value, pat)? {
                        return Ok(Some(pat_bindings));
                    }
                }
                Ok(None)
            },
            
            Pattern::And(patterns) => {
                for pat in patterns {
                    if let Some(pat_bindings) = self.match_pattern(value, pat)? {
                        bindings.extend(pat_bindings);
                    } else {
                        return Ok(None);
                    }
                }
                Ok(Some(bindings))
            },
        }
    }
    
    fn match_error(&self, error: &RuntimeError, pattern: &Pattern) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        let mut bindings = HashMap::new();
        
        match (error, pattern) {
            (RuntimeError::Error { message, code, confidence, context }, Pattern::Variable(name)) => {
                bindings.insert(name.clone(), Value::Error(RuntimeError::Error {
                    message: message.clone(),
                    code: code.clone(),
                    confidence: *confidence,
                    context: context.clone(),
                }));
                Ok(Some(bindings))
            },
            
            (RuntimeError::Throw(value), pattern) => {
                // Match thrown value against pattern
                if let Some(mut value_bindings) = self.match_pattern(value, pattern)? {
                    bindings.extend(value_bindings);
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            },
            
            _ => Ok(None),
        }
    }
    
    pub fn execute_module(&mut self, module: &Module, instance: &mut ModuleInstance) -> Result<(), RuntimeError> {
        // Set module confidence
        instance.confidence = module.confidence;
        
        // Process imports
        for import in &module.imports {
            self.process_import(import, instance)?;
        }
        
        // Execute module body
        for stmt in &module.body {
            self.eval_stmt(stmt)?;
        }
        
        // Process exports
        for export in &module.exports {
            self.process_export(export, instance)?;
        }
        
        Ok(())
    }
    
    fn process_import(&mut self, import: &Import, current_module: &ModuleInstance) -> Result<(), RuntimeError> {
        match import {
            Import::All { from, confidence } => {
                let module = self.module_loader.load_module(from, self)?;
                
                // Apply confidence adjustment if specified
                let exports = if let Some(conf) = confidence {
                    module.exports.iter().map(|(k, v)| {
                        (k.clone(), v.with_confidence(*conf))
                    }).collect()
                } else {
                    module.exports
                };
                
                self.variables.extend(exports);
            },
            
            Import::Named { names, from, confidence } => {
                let module = self.module_loader.load_module(from, self)?;
                
                for name in names {
                    if let Some(value) = module.exports.get(name) {
                        let value = if let Some(conf) = confidence {
                            value.with_confidence(*conf)
                        } else {
                            value.clone()
                        };
                        self.variables.insert(name.clone(), value);
                    } else {
                        return Err(RuntimeError::Error {
                            message: format!("Export '{}' not found in module '{}'", name, from),
                            code: Some("EXPORT_NOT_FOUND".to_string()),
                            confidence: Some(1.0),
                            context: None,
                        });
                    }
                }
            },
            
            Import::Aliased { name, as_name, from, confidence } => {
                let module = self.module_loader.load_module(from, self)?;
                
                if let Some(value) = module.exports.get(name) {
                    let value = if let Some(conf) = confidence {
                        value.with_confidence(*conf)
                    } else {
                        value.clone()
                    };
                    self.variables.insert(as_name.clone(), value);
                } else {
                    return Err(RuntimeError::Error {
                        message: format!("Export '{}' not found in module '{}'", name, from),
                        code: Some("EXPORT_NOT_FOUND".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    });
                }
            },
        }
        
        Ok(())
    }
    
    fn process_export(&mut self, export: &Export, instance: &mut ModuleInstance) -> Result<(), RuntimeError> {
        match export {
            Export::Named(name) => {
                if let Some(value) = self.variables.get(name) {
                    let value = if let Some(conf) = instance.confidence {
                        value.with_confidence(conf)
                    } else {
                        value.clone()
                    };
                    instance.exports.insert(name.clone(), value);
                } else {
                    return Err(RuntimeError::Error {
                        message: format!("Cannot export undefined value '{}'", name),
                        code: Some("UNDEFINED_EXPORT".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    });
                }
            },
            
            Export::All => {
                for (name, value) in &self.variables {
                    let value = if let Some(conf) = instance.confidence {
                        value.with_confidence(conf)
                    } else {
                        value.clone()
                    };
                    instance.exports.insert(name.clone(), value);
                }
            },
            
            Export::Default(expr) => {
                let value = self.eval_expr(expr)?;
                let value = if let Some(conf) = instance.confidence {
                    value.with_confidence(conf)
                } else {
                    value
                };
                instance.exports.insert("default".to_string(), value);
            },
        }
        
        Ok(())
    }
    
    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            // ... existing match arms ...
            
            Stmt::StructDefinition { name, type_parameters, confidence, fields, methods } => {
                if type_parameters.is_empty() {
                    // Handle non-generic struct as before
                    // ... existing code ...
                } else {
                    // Store generic struct definition
                    let mut field_map = HashMap::new();
                    let mut method_map = HashMap::new();
                    
                    for field in fields {
                        field_map.insert(field.name.clone(), field.clone());
                    }
                    
                    for method in methods {
                        method_map.insert(method.name.clone(), method.clone());
                    }
                    
                    self.generic_struct_definitions.insert(name.clone(), GenericStructDefinition {
                        type_parameters: type_parameters.clone(),
                        confidence: *confidence,
                        fields: field_map,
                        methods: method_map,
                    });
                }
                
                Ok(Value::Void)
            },
            
            Stmt::TraitDefinition { name, type_parameters, confidence, methods } => {
                if type_parameters.is_empty() {
                    // Handle non-generic trait as before
                    // ... existing code ...
                } else {
                    // Store generic trait definition
                    let mut method_map = HashMap::new();
                    
                    for method in methods {
                        method_map.insert(method.name.clone(), method.clone());
                    }
                    
                    self.generic_trait_definitions.insert(name.clone(), GenericTraitDefinition {
                        type_parameters: type_parameters.clone(),
                        confidence: *confidence,
                        methods: method_map,
                    });
                }
                
                Ok(Value::Void)
            },
            
            // ... rest of existing match arms ...
        }
    }
    
    fn instantiate_generic_struct(&mut self, name: &str, type_args: &[Type]) -> Result<StructDefinition, RuntimeError> {
        let generic_def = self.generic_struct_definitions.get(name)
            .ok_or_else(|| RuntimeError::Error {
                message: format!("Generic struct '{}' not found", name),
                code: Some("TYPE_ERROR".to_string()),
                confidence: Some(1.0),
                context: None,
            })?;
        
        if type_args.len() != generic_def.type_parameters.len() {
            return Err(RuntimeError::Error {
                message: format!(
                    "Wrong number of type arguments for '{}' (expected {}, got {})",
                    name,
                    generic_def.type_parameters.len(),
                    type_args.len()
                ),
                code: Some("TYPE_ERROR".to_string()),
                confidence: Some(1.0),
                context: None,
            });
        }
        
        // Create new type environment for instantiation
        let mut type_env = TypeEnvironment::new();
        
        // Bind type arguments to parameters
        for (param, arg) in generic_def.type_parameters.iter().zip(type_args) {
            type_env.bind_type(param.name.clone(), arg.clone());
            
            // Check trait bounds
            type_env.check_bounds(arg, &param.bounds)?;
            
            // Apply confidence bound if specified
            if let Some(conf_bound) = param.confidence_bound {
                type_env.bind_confidence(param.name.clone(), conf_bound);
            }
        }
        
        // Instantiate fields with concrete types
        let mut fields = Vec::new();
        for (name, field) in &generic_def.fields {
            fields.push(StructField {
                name: name.clone(),
                type_annotation: type_env.resolve_type(&field.type_annotation)?,
                confidence: field.confidence,
                default_value: field.default_value.clone(),
            });
        }
        
        // Instantiate methods with concrete types
        let mut methods = Vec::new();
        for (name, method) in &generic_def.methods {
            methods.push(StructMethod {
                name: name.clone(),
                type_parameters: method.type_parameters.clone(),
                parameters: method.parameters.iter()
                    .map(|(name, ty)| Ok((name.clone(), type_env.resolve_type(ty)?)))
                    .collect::<Result<_, RuntimeError>>()?,
                return_type: match &method.return_type {
                    Some(ty) => Some(type_env.resolve_type(ty)?),
                    None => None,
                },
                confidence: method.confidence,
                body: method.body.clone(),
            });
        }
        
        Ok(StructDefinition {
            confidence: generic_def.confidence,
            fields: fields.into_iter()
                .map(|f| (f.name.clone(), f))
                .collect(),
            methods: methods.into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
        })
    }

    fn find_operator_definition(&self, op: &Operator, lhs: &Value, rhs: &Value) -> Option<&OperatorDefinition> {
        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();
        self.operator_definitions.get(&(lhs_type, op.clone(), Some(rhs_type)))
    }

    fn find_unary_operator_definition(&self, op: &Operator, value: &Value) -> Option<&OperatorDefinition> {
        let value_type = value.get_type();
        self.operator_definitions.get(&(value_type, op.clone(), None))
    }

    fn eval_custom_operator(&mut self, def: &OperatorDefinition, lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
        // Save current scope
        let old_scope = self.variables.clone();
        
        // Set up operator scope
        self.variables.clear();
        self.variables.insert("lhs".to_string(), lhs);
        self.variables.insert("rhs".to_string(), rhs);
        
        // Execute operator body
        let mut result = Value::Void;
        for stmt in &def.body {
            result = self.eval(stmt)?;
        }
        
        // Apply operator confidence if specified
        if let Some(conf) = def.confidence {
            result = result.with_confidence(conf);
        }
        
        // Restore original scope
        self.variables = old_scope;
        
        Ok(result)
    }

    fn eval_custom_unary_operator(&mut self, def: &OperatorDefinition, value: Value) -> Result<Value, RuntimeError> {
        // Save current scope
        let old_scope = self.variables.clone();
        
        // Set up operator scope
        self.variables.clear();
        self.variables.insert("value".to_string(), value);
        
        // Execute operator body
        let mut result = Value::Void;
        for stmt in &def.body {
            result = self.eval(stmt)?;
        }
        
        // Apply operator confidence if specified
        if let Some(conf) = def.confidence {
            result = result.with_confidence(conf);
        }
        
        // Restore original scope
        self.variables = old_scope;
        
        Ok(result)
    }
}

pub struct AsyncInterpreter {
    interpreter: Interpreter,
    runtime: Runtime,
}

impl AsyncInterpreter {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            runtime: Runtime::new().unwrap(),
        }
    }
    
    pub async fn eval_async(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::AsyncFunctionDefinition { name, type_parameters, parameters, return_type, confidence, body } => {
                // Store async function definition
                let func = AsyncFunction {
                    type_parameters: type_parameters.clone(),
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    confidence: *confidence,
                    body: body.clone(),
                };
                self.interpreter.async_functions.insert(name.clone(), func);
                Ok(Value::Void)
            },
            
            Stmt::AsyncBlock { body, confidence } => {
                let mut result = Value::Void;
                for stmt in body {
                    result = self.eval_async(stmt).await?;
                }
                if let Some(conf) = confidence {
                    result = result.with_confidence(*conf);
                }
                Ok(result)
            },
            
            // Forward other statements to regular interpreter
            _ => self.interpreter.eval(stmt),
        }
    }
    
    pub async fn eval_expr_async(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Await { expr, confidence } => {
                let promise = self.eval_expr_async(expr).await?;
                let promise_state = promise.as_promise()?;
                
                // Create future that resolves when promise is settled
                let future = PromiseFuture {
                    promise: promise_state,
                };
                
                // Await the future
                let result = future.await?;
                
                // Apply confidence adjustment if specified
                if let Some(conf) = confidence {
                    Ok(result.with_confidence(*conf))
                } else {
                    Ok(result)
                }
            },
            
            Expr::AsyncCall { function, arguments, confidence } => {
                let func_value = self.eval_expr_async(function).await?;
                
                match func_value {
                    Value::AsyncFunction(func) => {
                        // Create new scope for function execution
                        let old_scope = self.interpreter.variables.clone();
                        self.interpreter.variables.clear();
                        
                        // Evaluate arguments
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            arg_values.push(self.eval_expr_async(arg).await?);
                        }
                        
                        // Bind arguments to parameters
                        for ((name, _), value) in func.parameters.iter().zip(arg_values) {
                            self.interpreter.variables.insert(name.clone(), value);
                        }
                        
                        // Execute function body
                        let mut result = Value::Void;
                        for stmt in &func.body {
                            result = self.eval_async(stmt).await?;
                        }
                        
                        // Apply function confidence
                        if let Some(func_conf) = func.confidence {
                            result = result.with_confidence(func_conf);
                        }
                        
                        // Apply call-site confidence
                        if let Some(call_conf) = confidence {
                            result = result.with_confidence(call_conf);
                        }
                        
                        // Restore original scope
                        self.interpreter.variables = old_scope;
                        
                        Ok(result)
                    },
                    _ => Err(RuntimeError::Error {
                        message: "Not an async function".to_string(),
                        code: Some("TYPE_ERROR".to_string()),
                        confidence: Some(1.0),
                        context: None,
                    }),
                }
            },
            
            Expr::Promise { value, confidence } => {
                // Create new promise
                let promise = Arc::new(Mutex::new(PromiseState::Pending(confidence.clone())));
                let promise_clone = promise.clone();
                
                // Spawn task to resolve promise
                self.runtime.spawn(async move {
                    match self.eval_expr_async(value).await {
                        Ok(result) => {
                            let mut state = promise_clone.lock().unwrap();
                            *state = PromiseState::Resolved(result);
                        },
                        Err(error) => {
                            let mut state = promise_clone.lock().unwrap();
                            *state = PromiseState::Rejected(error);
                        },
                    }
                });
                
                Ok(Value::Promise(promise))
            },
            
            // Forward other expressions to regular interpreter
            _ => self.interpreter.eval_expr(expr),
        }
    }
}

// Future implementation for awaiting promises
struct PromiseFuture {
    promise: Arc<Mutex<PromiseState>>,
}

impl Future for PromiseFuture {
    type Output = Result<Value, RuntimeError>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.promise.lock().unwrap();
        match &*state {
            PromiseState::Pending(_) => {
                // Register waker and return pending
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            PromiseState::Resolved(value) => Poll::Ready(Ok(value.clone())),
            PromiseState::Rejected(error) => Poll::Ready(Err(error.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt};

    #[test]
    fn test_confidence_declaration() {
        let mut interpreter = Interpreter::new();
        
        let stmt = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        
        let result = interpreter.eval(&stmt).unwrap();
        assert_eq!(result, Value::Float(0.8));
    }

    #[test]
    fn test_confidence_flow() {
        let mut interpreter = Interpreter::new();
        
        // First declare x = 0.8
        let decl = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        interpreter.eval(&decl).unwrap();
        
        // Then do x ~> 0.7
        let flow = Stmt::Expression(Expr::ConfidenceFlow {
            source: Box::new(Expr::Identifier("x".to_string())),
            target: Box::new(Expr::Float(0.7)),
        });
        
        let result = interpreter.eval(&flow).unwrap();
        match result {
            Value::Float(f) => {
                let expected = 0.8 * 0.7;
                let diff = (f - expected).abs();
                assert!(diff < 1e-10, "Expected approximately {}, got {}", expected, f);
            },
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_function_definition_and_call() {
        let mut interpreter = Interpreter::new();
        
        // Define a function that adds two numbers
        let func_def = Stmt::FunctionDefinition {
            name: "add".to_string(),
            parameters: vec!["x".to_string(), "y".to_string()],
            body: vec![
                Stmt::Expression(Expr::Return(Some(Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expr::Identifier("y".to_string())),
                })))),
            ],
            confidence_level: Some(0.9),
        };
        
        interpreter.eval(&func_def).unwrap();
        
        // Call the function
        let call = Expr::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                Expr::Float(2.0),
                Expr::Float(3.0),
            ],
        };
        
        let result = interpreter.eval_expr(&call).unwrap();
        match result {
            Value::Float(f) => {
                let expected = (2.0 + 3.0) * 0.9;
                let diff = (f - expected).abs();
                assert!(diff < 1e-10, "Expected {}, got {}", expected, f);
            },
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_function_return() {
        let mut interpreter = Interpreter::new();
        
        // Define a function that returns early
        let func_def = Stmt::FunctionDefinition {
            name: "early_return".to_string(),
            parameters: vec![],
            body: vec![
                Stmt::Expression(Expr::Return(Some(Box::new(Expr::Float(1.0))))),
                Stmt::Expression(Expr::Float(2.0)),
            ],
            confidence_level: None,
        };
        
        interpreter.eval(&func_def).unwrap();
        
        // Call the function
        let call = Expr::FunctionCall {
            name: "early_return".to_string(),
            arguments: vec![],
        };
        
        let result = interpreter.eval_expr(&call).unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_array_operations() {
        let mut interpreter = Interpreter::new();
        
        // Test array creation
        let array_expr = Expr::Array(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ]);
        let result = interpreter.eval_expr(&array_expr).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        
        // Test array access
        let access_expr = Expr::ArrayAccess {
            array: Box::new(array_expr.clone()),
            index: Box::new(Expr::Integer(1)),
        };
        let result = interpreter.eval_expr(&access_expr).unwrap();
        assert_eq!(result, Value::Integer(2));
        
        // Test array slice
        let slice_expr = Expr::ArraySlice {
            array: Box::new(array_expr.clone()),
            start: Some(Box::new(Expr::Integer(1))),
            end: Some(Box::new(Expr::Integer(3))),
        };
        let result = interpreter.eval_expr(&slice_expr).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::Integer(2),
            Value::Integer(3),
        ]));
        
        // Test array length
        let length_expr = Expr::ArrayLength(Box::new(array_expr.clone()));
        let result = interpreter.eval_expr(&length_expr).unwrap();
        assert_eq!(result, Value::Integer(3));
        
        // Test array push
        let push_expr = Expr::ArrayPush {
            array: Box::new(array_expr.clone()),
            value: Box::new(Expr::Integer(4)),
        };
        let result = interpreter.eval_expr(&push_expr).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]));
    }

    #[test]
    fn test_for_loop() {
        let mut interpreter = Interpreter::new();
        
        // Test C-style for loop
        let for_loop = Stmt::ForLoop {
            initializer: Some(Box::new(Stmt::Declaration {
                name: "i".to_string(),
                value: Expr::Integer(0),
            })),
            condition: Some(Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Identifier("i".to_string())),
                op: BinaryOperator::LessThan,
                right: Box::new(Expr::Integer(3)),
            })),
            increment: Some(Box::new(Expr::Assignment {
                target: "i".to_string(),
                value: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier("i".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expr::Integer(1)),
                }),
            })),
            body: vec![
                Stmt::Expression(Expr::Identifier("i".to_string())),
            ],
        };
        
        let result = interpreter.eval(&for_loop).unwrap();
        assert_eq!(result, Value::Integer(2));
    }
    
    #[test]
    fn test_for_in_loop() {
        let mut interpreter = Interpreter::new();
        
        // Test for-in loop with array
        let array = Expr::Array(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ]);
        
        let for_in_loop = Stmt::ForInLoop {
            variable: "x".to_string(),
            iterator: Box::new(array),
            body: vec![
                Stmt::Expression(Expr::Identifier("x".to_string())),
            ],
        };
        
        let result = interpreter.eval(&for_in_loop).unwrap();
        assert_eq!(result, Value::Integer(3));
    }
    
    #[test]
    fn test_while_loop() {
        let mut interpreter = Interpreter::new();
        
        // Test while loop
        let while_loop = Stmt::WhileLoop {
            condition: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Identifier("i".to_string())),
                op: BinaryOperator::LessThan,
                right: Box::new(Expr::Integer(3)),
            }),
            body: vec![
                Stmt::Assignment {
                    target: "i".to_string(),
                    value: Expr::BinaryOp {
                        left: Box::new(Expr::Identifier("i".to_string())),
                        op: BinaryOperator::Add,
                        right: Box::new(Expr::Integer(1)),
                    },
                },
            ],
        };
        
        interpreter.variables.insert("i".to_string(), Value::Integer(0));
        let result = interpreter.eval(&while_loop).unwrap();
        assert_eq!(interpreter.variables.get("i").unwrap(), &Value::Integer(3));
    }
    
    #[test]
    fn test_break_continue() {
        let mut interpreter = Interpreter::new();
        
        // Test break and continue
        let loop_with_break = Stmt::WhileLoop {
            condition: Box::new(Expr::Float(1.0)),
            body: vec![
                Stmt::Break,
            ],
        };
        
        interpreter.eval(&loop_with_break).unwrap();
        
        let loop_with_continue = Stmt::WhileLoop {
            condition: Box::new(Expr::Identifier("i".to_string())),
            body: vec![
                Stmt::Continue,
                Stmt::Expression(Expr::String("unreachable".to_string())),
            ],
        };
        
        interpreter.variables.insert("i".to_string(), Value::Float(0.0));
        interpreter.eval(&loop_with_continue).unwrap();
    }
    
    #[test]
    fn test_pattern_matching() {
        let mut interpreter = Interpreter::new();
        
        // Test basic pattern matching
        let match_expr = Expr::Match {
            value: Box::new(Expr::Array(vec![
                Expr::Float(0.8),
                Expr::Float(0.6),
                Expr::Float(0.9),
            ])),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Array(vec![
                        Pattern::Confidence {
                            value: Box::new(Pattern::Variable("high".to_string())),
                            range: (0.8, 1.0),
                        },
                        Pattern::Variable("mid".to_string()),
                        Pattern::Confidence {
                            value: Box::new(Pattern::Variable("very_high".to_string())),
                            range: (0.9, 1.0),
                        },
                    ]),
                    guard: None,
                    body: vec![
                        Stmt::Expression(Expr::Identifier("very_high".to_string())),
                    ],
                },
            ],
        };
        
        let result = interpreter.eval_expr(&match_expr).unwrap();
        assert_eq!(result, Value::Float(0.9));
        
        // Test pattern matching with guard
        let match_with_guard = Expr::Match {
            value: Box::new(Expr::Float(0.85)),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Variable("x".to_string()),
                    guard: Some(Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        op: BinaryOperator::GreaterThan,
                        right: Box::new(Expr::Float(0.8)),
                    })),
                    body: vec![
                        Stmt::Expression(Expr::Identifier("x".to_string())),
                    ],
                },
            ],
        };
        
        let result = interpreter.eval_expr(&match_with_guard).unwrap();
        assert_eq!(result, Value::Float(0.85));
    }
    
    #[test]
    fn test_context_pattern_matching() {
        let mut interpreter = Interpreter::new();
        
        // Create a context value
        let ctx_value = Value::Context(Context::new(
            "validation".to_string(),
            Value::Float(0.9),
            vec![],
            vec![],
        ));
        
        // Test context pattern matching
        let match_expr = Expr::Match {
            value: Box::new(Expr::Value(ctx_value)),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Context {
                        name: "validation".to_string(),
                        pattern: Box::new(Pattern::Confidence {
                            value: Box::new(Pattern::Variable("conf".to_string())),
                            range: (0.8, 1.0),
                        }),
                    },
                    guard: None,
                    body: vec![
                        Stmt::Expression(Expr::Identifier("conf".to_string())),
                    ],
                },
            ],
        };
        
        let result = interpreter.eval_expr(&match_expr).unwrap();
        assert_eq!(result, Value::Float(0.9));
    }
    
    #[test]
    fn test_try_catch() {
        let mut interpreter = Interpreter::new();
        
        // Test basic try-catch
        let try_catch = Stmt::TryCatch {
            body: vec![
                Stmt::Expression(Expr::Throw {
                    error: Box::new(Expr::Error {
                        message: "Test error".to_string(),
                        code: Some("TEST001".to_string()),
                        confidence: Some(0.8),
                        context: None,
                    }),
                    confidence: None,
                }),
            ],
            catches: vec![
                CatchClause {
                    pattern: Pattern::Variable("e".to_string()),
                    guard: None,
                    body: vec![
                        Stmt::Expression(Expr::Identifier("e".to_string())),
                    ],
                    confidence_adjustment: Some(0.9),
                },
            ],
            finally: None,
        };
        
        let result = interpreter.eval_stmt(&try_catch).unwrap();
        match result {
            Value::Error(RuntimeError::Error { confidence, .. }) => {
                assert_eq!(confidence, Some(0.9));
            },
            _ => panic!("Expected error value"),
        }
    }
    
    #[test]
    fn test_error_confidence() {
        let mut interpreter = Interpreter::new();
        
        // Test error with confidence adjustment
        let throw_expr = Expr::Throw {
            error: Box::new(Expr::String("Test error".to_string())),
            confidence: Some(Box::new(Expr::Float(0.8))),
        };
        
        let result = interpreter.eval_expr(&throw_expr);
        match result {
            Err(RuntimeError::Throw(value)) => {
                assert_eq!(value.confidence(), Some(0.8));
            },
            _ => panic!("Expected throw with confidence"),
        }
    }
    
    #[test]
    fn test_struct_definition_and_instantiation() {
        let mut interpreter = Interpreter::new();
        
        // Define a struct
        let struct_def = Stmt::StructDefinition {
            name: "Person".to_string(),
            type_parameters: vec![],
            confidence: Some(0.9),
            fields: vec![
                StructField {
                    name: "name".to_string(),
                    type_annotation: Type::Simple("String".to_string()),
                    confidence: Some(0.95),
                    default_value: None,
                },
                StructField {
                    name: "age".to_string(),
                    type_annotation: Type::Simple("i64".to_string()),
                    confidence: Some(0.8),
                    default_value: None,
                },
            ],
            methods: vec![
                StructMethod {
                    name: "is_adult".to_string(),
                    type_parameters: vec![],
                    parameters: vec![],
                    return_type: Some(Type::Simple("bool".to_string())),
                    confidence: Some(0.9),
                    body: vec![
                        Stmt::Expression(Expr::BinaryOp {
                            left: Box::new(Expr::FieldAccess {
                                object: Box::new(Expr::This),
                                field: "age".to_string(),
                            }),
                            op: BinaryOperator::GreaterThanOrEqual,
                            right: Box::new(Expr::Integer(18)),
                        }),
                    ],
                },
            ],
        };
        
        interpreter.eval_stmt(&struct_def).unwrap();
        
        // Create an instance
        let instance = interpreter.instantiate_generic_struct(
            "Person",
            &[],
        ).unwrap();
        
        assert_eq!(instance.confidence, Some(0.9));
        
        let name_field = instance.fields.get("name").unwrap();
        assert_eq!(name_field.type_annotation, Type::Simple("String".to_string()));
        assert_eq!(name_field.confidence, Some(0.95));
        
        let age_field = instance.fields.get("age").unwrap();
        assert_eq!(age_field.type_annotation, Type::Simple("i64".to_string()));
        assert_eq!(age_field.confidence, Some(0.8));
        
        // Call method
        let method_call = Expr::MethodCall {
            object: Box::new(instance),
            method: "is_adult".to_string(),
            arguments: vec![],
        };
        
        let result = interpreter.eval_expr(&method_call).unwrap();
        match result {
            Value::Float(f) => assert!(f > 0.0),  // Should be true with confidence
            _ => panic!("Expected boolean result"),
        }
    }
    
    #[test]
    fn test_trait_definition_and_implementation() {
        let mut interpreter = Interpreter::new();
        
        // Define a trait
        let trait_def = Stmt::TraitDefinition {
            name: "Validator".to_string(),
            type_parameters: vec![],
            confidence: Some(0.9),
            methods: vec![
                TraitMethod {
                    name: "validate".to_string(),
                    type_parameters: vec![],
                    parameters: vec!["context".to_string()],
                    body: None,  // Abstract method
                },
            ],
        };
        
        interpreter.eval_stmt(&trait_def).unwrap();
        
        // Define a struct
        let struct_def = Stmt::StructDefinition {
            name: "Data".to_string(),
            type_parameters: vec![],
            confidence: Some(0.9),
            fields: vec![
                StructField {
                    name: "value".to_string(),
                    type_annotation: Type::Simple("f64".to_string()),
                    confidence: Some(0.95),
                    default_value: None,
                },
            ],
            methods: vec![],
        };
        
        interpreter.eval_stmt(&struct_def).unwrap();
        
        // Implement trait for struct
        let impl_def = Stmt::ImplTrait {
            trait_name: "Validator".to_string(),
            struct_name: "Data".to_string(),
            confidence: Some(0.9),
            methods: vec![
                StructMethod {
                    name: "validate".to_string(),
                    type_parameters: vec![],
                    parameters: vec!["context".to_string()],
                    return_type: Some(Type::Simple("bool".to_string())),
                    confidence: Some(0.95),
                    body: vec![
                        Stmt::Expression(Expr::BinaryOp {
                            left: Box::new(Expr::FieldAccess {
                                object: Box::new(Expr::This),
                                field: "value".to_string(),
                            }),
                            op: BinaryOperator::GreaterThan,
                            right: Box::new(Expr::Float(0.0)),
                        }),
                    ],
                },
            ],
        };
        
        interpreter.eval_stmt(&impl_def).unwrap();
        
        // Create instance and call trait method
        let instance = interpreter.instantiate_generic_struct(
            "Data",
            &[],
        ).unwrap();
        
        let method_call = Expr::DynamicMethodCall {
            object: Box::new(instance),
            trait_name: "Validator".to_string(),
            method: "validate".to_string(),
            arguments: vec![
                Expr::String("test".to_string()),
            ],
        };
        
        let result = interpreter.eval_expr(&method_call).unwrap();
        match result {
            Value::Float(f) => {
                assert!(f > 0.0);  // Should be true with confidence
                let conf = result.confidence().unwrap();
                assert!((conf - 0.9 * 0.95).abs() < 1e-10);  // Combined confidence
            },
            _ => panic!("Expected boolean result"),
        }
    }
} 