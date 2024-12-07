use std::sync::Arc;
use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::{PrismError, Result};
use crate::token::TokenKind;
use crate::value::{Value, ValueKind};
use crate::module::ModuleRegistry;
use std::sync::RwLock;
use std::path::PathBuf;
use std::fmt;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use crate::module::Module;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub struct Interpreter {
    environment: Arc<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Arc::new(Environment::new()),
        }
    }

    pub async fn evaluate(&self, source: String) -> Result<Value> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        
        let mut result = Value::new(ValueKind::Nil);
        
        for stmt in statements {
            result = self.execute_statement(&stmt, self.environment.clone()).await?;
        }
        
        Ok(result)
    }

    pub fn register_builtin_module(&self, name: &str, module: Arc<RwLock<Module>>) -> Result<()> {
        let registry = ModuleRegistry::new();
        registry.register_module(name, module)
    }

    pub async fn import_symbol(&self, module_name: &str, symbol: &str) -> Result<Value> {
        let registry = ModuleRegistry::new();
        registry.resolve_import(module_name, symbol)
    }

    pub fn load_module(&self, name: &str) -> Result<Arc<RwLock<Module>>> {
        let registry = ModuleRegistry::new();
        registry.load_module(name)
    }

    pub async fn execute(&self, statements: &[Stmt], env: Arc<Environment>) -> Result<Value> {
        let mut result = Value::new(ValueKind::Nil);
        
        for stmt in statements {
            result = self.execute_statement(stmt, env.clone()).await?;
        }
        
        Ok(result)
    }

    pub async fn execute_statement(&self, stmt: &Stmt, env: Arc<Environment>) -> Result<Value> {
        let result = match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr, env).await
            }
            Stmt::Let(name, init) => {
                let value = if let Some(init) = init {
                    self.evaluate_expression(init, env.clone()).await?
                } else {
                    Value::new(ValueKind::Nil)
                };
                env.define(name.clone(), value.clone());
                Ok(value)
            }
            Stmt::Block(statements) => {
                let block_env = Arc::new(Environment::with_enclosing(env));
                let mut result = Value::new(ValueKind::Nil);
                
                for stmt in statements {
                    result = Box::pin(self.execute_statement(stmt, block_env.clone())).await?;
                }
                
                Ok(result)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_value = self.evaluate_expression(condition, env.clone()).await?;
                
                if cond_value.is_truthy() {
                    Box::pin(self.execute_statement(then_branch, env)).await
                } else if let Some(else_stmt) = else_branch {
                    Box::pin(self.execute_statement(else_stmt, env)).await
                } else {
                    Ok(Value::new(ValueKind::Nil))
                }
            }
            Stmt::UncertainIf { condition, then_branch, medium_branch, low_branch } => {
                let cond_value = self.evaluate_expression(condition, env.clone()).await?;
                let confidence = cond_value.get_confidence().unwrap_or(1.0);
                
                if confidence >= 0.8 {
                    Box::pin(self.execute_statement(then_branch, env)).await
                } else if confidence >= 0.5 {
                    if let Some(medium) = medium_branch {
                        Box::pin(self.execute_statement(medium, env)).await
                    } else {
                        Ok(Value::new(ValueKind::Nil))
                    }
                } else {
                    if let Some(low) = low_branch {
                        Box::pin(self.execute_statement(low, env)).await
                    } else {
                        Ok(Value::new(ValueKind::Nil))
                    }
                }
            }
            Stmt::While { condition, body } => {
                let mut result = Value::new(ValueKind::Nil);
                
                while self.evaluate_expression(condition, env.clone()).await?.is_truthy() {
                    result = Box::pin(self.execute_statement(body, env.clone())).await?;
                }
                
                Ok(result)
            }
            Stmt::Function { name, params, body, is_async, confidence } => {
                let function = Value::new(ValueKind::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: Box::new(*body.clone()),
                    is_async: *is_async,
                    confidence: *confidence,
                });
                env.define(name.clone(), function.clone());
                Ok(function)
            }
            Stmt::Return(value) => {
                let value = if let Some(expr) = value {
                    self.evaluate_expression(expr, env).await?
                } else {
                    Value::new(ValueKind::Nil)
                };
                Ok(value)
            }
            Stmt::Context { name, body } => {
                let mut result = Box::pin(self.execute_statement(body, env)).await?;
                result.set_context(name.clone());
                Ok(result)
            }
            Stmt::Import { module, imports, confidence } => {
                // Load the module using ModuleRegistry
                let registry = ModuleRegistry::new();
                let imported_module = registry.load_module(&module)?;

                // Process each import and add to environment
                for (name, alias) in imports {
                    let symbol_name = alias.as_ref().map_or_else(|| name.clone(), |a| a.clone());
                    let value = registry.resolve_import(&module, &name)?;
                    
                    // Create a new value with potentially updated confidence
                    let mut imported_value = value.clone();
                    if let Some(conf) = confidence {
                        // Combine confidences if both exist
                        if let Some(existing_conf) = value.get_confidence() {
                            imported_value.set_confidence(existing_conf * conf);
                        } else {
                            imported_value.set_confidence(*conf);
                        }
                    }
                    
                    env.define(symbol_name, imported_value);
                }
                
                Ok(Value::new(ValueKind::Nil))
            }
            Stmt::Export(name, stmt) => {
                let value = Box::pin(self.execute_statement(stmt, env.clone())).await?;
                
                // Get the current module from environment context
                if let Some(current_module) = env.get_module() {
                    let mut module = current_module.write().unwrap();
                    module.register_export(&name, value.clone());
                }
                
                // Also define in current environment
                env.define(name.clone(), value.clone());
                Ok(value)
            }
            Stmt::Module { name, body, confidence } => {
                // Create a new module
                let module = Arc::new(RwLock::new(Module::new(&name, PathBuf::from("memory"))));
                
                // Create a new environment with the module context
                let mut module_env = Environment::with_enclosing(env.clone());
                module_env.set_module(module.clone());
                let module_env = Arc::new(module_env);

                // Execute module body
                let mut result = Value::new(ValueKind::Nil);
                for stmt in body {
                    result = Box::pin(self.execute_statement(&stmt, module_env.clone())).await?;
                }

                // Set confidence if provided
                if let Some(conf) = confidence {
                    result.set_confidence(*conf);
                    module.write().unwrap().set_confidence(*conf);
                }

                // Register module in the registry
                let registry = ModuleRegistry::new();
                registry.register_module(&name, module.clone())?;

                // Create module value
                let module_value = Value::new(ValueKind::Module(module));
                env.define(name.clone(), module_value.clone());

                Ok(module_value)
            }
        };
        result
    }

    pub async fn evaluate_expression(&self, expr: &Expr, env: Arc<Environment>) -> Result<Value> {
        let result = match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable(name) => {
                env.get(name).ok_or_else(|| Box::new(PrismError::UndefinedVariable(name.clone())) as Box<dyn std::error::Error + Send + Sync>)
            }
            Expr::Assign { name, value } => {
                let value = Box::pin(self.evaluate_expression(value, env.clone())).await?;
                env.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Binary { left, operator, right } => {
                let left = Box::pin(self.evaluate_expression(left, env.clone())).await?;
                let right = Box::pin(self.evaluate_expression(right, env.clone())).await?;
                self.evaluate_binary_op(&left, &operator.kind, &right)
            }
            Expr::Unary { operator, right } => {
                let right = Box::pin(self.evaluate_expression(right, env)).await?;
                self.evaluate_unary_op(&operator.kind, &right)
            }
            Expr::Call { callee, arguments } => {
                let callee = Box::pin(self.evaluate_expression(callee, env.clone())).await?;
                let mut evaluated_args = Vec::new();
                
                for arg in arguments {
                    evaluated_args.push(Box::pin(self.evaluate_expression(arg, env.clone())).await?);
                }
                
                Box::pin(self.call_function(&callee, evaluated_args)).await
            }
            Expr::Get { object, name } => {
                let object = Box::pin(self.evaluate_expression(object, env)).await?;
                self.get_property(&object, name)
            }
            Expr::Logical { left, operator, right } => {
                let left = Box::pin(self.evaluate_expression(left, env.clone())).await?;
                
                match operator.kind {
                    TokenKind::And if !left.is_truthy() => Ok(left),
                    TokenKind::Or if left.is_truthy() => Ok(left),
                    _ => Box::pin(self.evaluate_expression(right, env)).await,
                }
            }
            Expr::Confidence { expr, confidence } => {
                let mut value = Box::pin(self.evaluate_expression(expr, env)).await?;
                value.set_confidence(*confidence);
                Ok(value)
            }
            Expr::ConfidenceCombine { left, right } => {
                let left = Box::pin(self.evaluate_expression(left, env.clone())).await?;
                let right = Box::pin(self.evaluate_expression(right, env)).await?;
                self.combine_confidence(&left, &right)
            }
            Expr::InContext { context, body } => {
                let mut value = Box::pin(self.evaluate_expression(body, env)).await?;
                value.set_context(context.clone());
                Ok(value)
            }
            Expr::Grouping(expr) => Box::pin(self.evaluate_expression(expr, env)).await,
        };
        result
    }

    async fn call_function(&self, callee: &Value, arguments: Vec<Value>) -> Result<Value> {
        match &callee.kind {
            ValueKind::Function { name: _, params, body, is_async: _, confidence } => {
                if params.len() != arguments.len() {
                    return Err(Box::new(PrismError::InvalidArgument(format!(
                        "Expected {} arguments but got {}",
                        params.len(),
                        arguments.len()
                    ))) as Box<dyn std::error::Error + Send + Sync>);
                }

                let env = Arc::new(Environment::new());
                for (param, arg) in params.iter().zip(arguments) {
                    env.define(param.clone(), arg);
                }

                let mut result = Box::pin(self.execute_statement(body, env)).await?;
                if let Some(conf) = confidence {
                    result.set_confidence(*conf);
                }
                Ok(result)
            }
            ValueKind::NativeFunction(f) => {
                let result = f(arguments).await?;
                Ok(result)
            }
            _ => Err(Box::new(PrismError::TypeError(format!(
                "Can only call functions and classes. Got: {:?}",
                callee.kind
            ))) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    fn evaluate_binary_op(&self, left: &Value, operator: &TokenKind, right: &Value) -> Result<Value> {
        match (&left.kind, operator, &right.kind) {
            (ValueKind::Number(l), TokenKind::Plus, ValueKind::Number(r)) => Ok(Value::new(ValueKind::Number(l + r))),
            (ValueKind::Number(l), TokenKind::Minus, ValueKind::Number(r)) => Ok(Value::new(ValueKind::Number(l - r))),
            (ValueKind::Number(l), TokenKind::Star, ValueKind::Number(r)) => Ok(Value::new(ValueKind::Number(l * r))),
            (ValueKind::Number(l), TokenKind::Slash, ValueKind::Number(r)) => {
                if *r == 0.0 {
                    Err(Box::new(PrismError::InvalidOperation("Division by zero".to_string())) as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Ok(Value::new(ValueKind::Number(l / r)))
                }
            }
            (ValueKind::String(l), TokenKind::Plus, ValueKind::String(r)) => Ok(Value::new(ValueKind::String(format!("{}{}", l, r)))),
            _ => Err(Box::new(PrismError::TypeError("Invalid operands for binary operator".to_string())) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    fn evaluate_unary_op(&self, operator: &TokenKind, right: &Value) -> Result<Value> {
        match (operator, &right.kind) {
            (TokenKind::Minus, ValueKind::Number(n)) => Ok(Value::new(ValueKind::Number(-n))),
            (TokenKind::Bang, _) => Ok(Value::new(ValueKind::Boolean(!right.is_truthy()))),
            _ => Err(Box::new(PrismError::TypeError("Invalid operand for unary operator".to_string())) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    fn get_property(&self, object: &Value, _name: &str) -> Result<Value> {
        match &object.kind {
            ValueKind::Object(_obj) => {
                // Implementation for object property access
                Err(Box::new(PrismError::Runtime("Object property access not implemented".to_string())) as Box<dyn std::error::Error + Send + Sync>)
            }
            _ => Err(Box::new(PrismError::TypeError("Only objects have properties".to_string())) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    fn combine_confidence(&self, left: &Value, right: &Value) -> Result<Value> {
        let left_conf = left.get_confidence().unwrap_or(1.0);
        let right_conf = right.get_confidence().unwrap_or(1.0);
        let mut result = left.clone();
        result.set_confidence(left_conf * right_conf);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_module_definition_and_export() {
        let source = r#"
            module math ~> 0.9 {
                export fn add(a, b) {
                    return a + b;
                }
                
                export let PI = 3.14159;
            }
        "#;

        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        
        let interpreter = Interpreter::new();
        let env = Arc::new(Environment::new());
        
        // Execute module definition
        for stmt in statements {
            interpreter.execute_statement(&stmt, env.clone()).await.unwrap();
        }
        
        // Verify module was created and exports are accessible
        if let Some(Value { kind: ValueKind::Module(module), .. }) = env.get("math") {
            let module = module.read().unwrap();
            assert!(module.get_export("add").is_some());
            assert!(module.get_export("PI").is_some());
            assert_eq!(module.confidence, Some(0.9));
        } else {
            panic!("Module not found or wrong type");
        }
    }

    #[tokio::test]
    async fn test_module_import() {
        // First create a module file
        std::fs::create_dir_all("lib").unwrap();
        std::fs::write(
            "lib/utils.prism",
            r#"
                export fn multiply(a, b) {
                    return a * b;
                }
                
                export let FACTOR = 2;
            "#,
        ).unwrap();

        let source = r#"
            import { multiply as mul, FACTOR } from "utils" ~> 0.8;
            
            let result = mul(FACTOR, 3);
        "#;

        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        
        let interpreter = Interpreter::new();
        let env = Arc::new(Environment::new());
        
        // Execute import and usage
        for stmt in statements {
            interpreter.execute_statement(&stmt, env.clone()).await.unwrap();
        }
        
        // Verify imported values are accessible
        assert!(env.get("mul").is_some());
        assert!(env.get("FACTOR").is_some());
        
        if let Some(Value { kind: ValueKind::Number(result), .. }) = env.get("result") {
            assert_eq!(result, 6.0); // 2 * 3
        } else {
            panic!("Result not found or wrong type");
        }

        // Clean up
        std::fs::remove_file("lib/utils.prism").unwrap();
        std::fs::remove_dir("lib").unwrap();
    }

    #[tokio::test]
    async fn test_module_confidence_propagation() {
        let source = r#"
            module calculator ~> 0.9 {
                export fn add(a, b) ~> 0.95 {
                    return a + b;
                }
            }

            import { add } from "calculator" ~> 0.8;
            let result = add(2, 3);
        "#;

        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        
        let interpreter = Interpreter::new();
        let env = Arc::new(Environment::new());
        
        // Execute statements
        for stmt in statements {
            interpreter.execute_statement(&stmt, env.clone()).await.unwrap();
        }
        
        // Verify confidence propagation
        if let Some(Value { kind: ValueKind::Number(result), confidence, .. }) = env.get("result") {
            assert_eq!(result, 5.0);
            // Confidence should be the product of all confidences in the chain
            assert_eq!(confidence, Some(0.9 * 0.95 * 0.8));
        } else {
            panic!("Result not found or wrong type");
        }
    }
}
