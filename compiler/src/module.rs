use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use crate::ast::Stmt;
use crate::environment::Environment;
use crate::error::{PrismError, Result};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::value::{Value, ValueKind};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub exports: HashMap<String, Value>,
    pub imports: HashMap<String, (String, String)>, // (module_name, symbol_name)
    pub statements: Vec<Stmt>,
    pub is_loaded: bool,
    pub confidence: Option<f64>,
}

impl Module {
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path,
            exports: HashMap::new(),
            imports: HashMap::new(),
            statements: Vec::new(),
            is_loaded: false,
            confidence: None,
        }
    }

    pub fn register_export(&mut self, name: &str, value: Value) {
        self.exports.insert(name.to_string(), value);
    }

    pub fn register_import(&mut self, local_name: &str, module: &str, symbol: &str) {
        self.imports.insert(
            local_name.to_string(),
            (module.to_string(), symbol.to_string()),
        );
    }

    pub fn add_statement(&mut self, stmt: Stmt) {
        self.statements.push(stmt);
    }

    pub fn set_confidence(&mut self, confidence: f64) {
        self.confidence = Some(confidence);
    }

    pub fn get_export(&self, name: &str) -> Option<&Value> {
        self.exports.get(name)
    }

    pub fn get_import(&self, name: &str) -> Option<&(String, String)> {
        self.imports.get(name)
    }

    pub fn mark_loaded(&mut self) {
        self.is_loaded = true;
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub fn get_statements(&self) -> &[Stmt] {
        &self.statements
    }
}

pub struct ModuleRegistry {
    modules: RwLock<HashMap<String, Arc<RwLock<Module>>>>,
    module_cache: RwLock<HashMap<PathBuf, Arc<RwLock<Module>>>>,
    search_paths: Vec<PathBuf>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
            module_cache: RwLock::new(HashMap::new()),
            search_paths: vec![PathBuf::from("std"), PathBuf::from("lib")],
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    pub fn register_module(&self, name: &str, module: Arc<RwLock<Module>>) -> Result<()> {
        self.modules.write().unwrap().insert(name.to_string(), module);
        Ok(())
    }

    pub fn load_module(&self, name: &str) -> Result<Arc<RwLock<Module>>> {
        // Check cache first
        if let Some(module) = self.modules.read().unwrap().get(name) {
            return Ok(module.clone());
        }

        // Search in paths
        for path in &self.search_paths {
            let module_path = path.join(name).with_extension("prism");
            if module_path.exists() {
                // Read and parse module file
                let source = std::fs::read_to_string(&module_path)?;
                let module = self.parse_and_create_module(name, &source, module_path.clone())?;
                
                // Cache the module
                let module = Arc::new(RwLock::new(module));
                self.modules.write().unwrap().insert(name.to_string(), module.clone());
                self.module_cache.write().unwrap().insert(module_path, module.clone());
                
                return Ok(module);
            }
        }

        Err(Box::new(PrismError::ModuleNotFound(name.to_string())))
    }

    fn parse_and_create_module(&self, name: &str, source: &str, path: PathBuf) -> Result<Module> {
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        let mut module = Module::new(name, path);
        
        // Process module statements
        for stmt in statements {
            match stmt {
                Stmt::Export(name, stmt) => {
                    // Handle export statement
                    let value = match *stmt {
                        Stmt::Function { name, params, body, is_async, confidence } => {
                            Value::new(ValueKind::Function {
                                name: name.clone(),
                                params,
                                body: Box::new(*body),
                                is_async,
                                confidence,
                            })
                        }
                        Stmt::Expression(expr) => {
                            // Evaluate expression to a value
                            let interpreter = Interpreter::new();
                            let env = Arc::new(Environment::new());
                            tokio::runtime::Runtime::new()
                                .unwrap()
                                .block_on(interpreter.evaluate_expression(&expr, env))?
                        }
                        _ => return Err(Box::new(PrismError::Runtime("Invalid export statement".to_string())))
                    };
                    module.register_export(&name, value);
                }
                Stmt::Import { module: mod_name, imports, confidence: _ } => {
                    // Handle import statement
                    for (name, alias) in imports {
                        let alias = alias.unwrap_or_else(|| name.clone());
                        module.register_import(&alias, &mod_name, &name);
                    }
                }
                _ => {
                    // Other statements are added to module body
                    module.add_statement(stmt);
                }
            }
        }

        Ok(module)
    }

    pub fn resolve_import(&self, module_name: &str, symbol: &str) -> Result<Value> {
        let module = self.load_module(module_name)?;
        let module_guard = module.read().unwrap();
        
        if let Some(value) = module_guard.get_export(symbol) {
            Ok(value.clone())
        } else {
            Err(Box::new(PrismError::SymbolNotFound {
                module: module_name.to_string(),
                symbol: symbol.to_string(),
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: String,
    pub symbols: Vec<(String, String)>, // (local_name, original_name)
}

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub value: Value,
}

impl Import {
    pub fn new(module_name: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
            symbols: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, local_name: &str, original_name: &str) {
        self.symbols.push((local_name.to_string(), original_name.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_loading() {
        let registry = ModuleRegistry::new();
        
        // Create test module
        std::fs::create_dir_all("lib").unwrap();
        std::fs::write(
            "lib/test_module.prism",
            r#"
                export fn add(a, b) {
                    return a + b;
                }

                export let greeting = "Hello, World!" ~> 0.9;
            "#,
        ).unwrap();

        // Load module
        let module = registry.load_module("test_module").unwrap();
        
        // Verify exports
        assert!(module.read().unwrap().get_export("add").is_some());
        assert!(module.read().unwrap().get_export("greeting").is_some());
        
        // Clean up
        std::fs::remove_file("lib/test_module.prism").unwrap();
        std::fs::remove_dir("lib").unwrap();
    }

    #[tokio::test]
    async fn test_module_imports() {
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
        
        // Verify imports
        let module = env.get_module().unwrap();
        let module_guard = module.read().unwrap();
        assert!(module_guard.get_import("multiply").is_some());
        let (module_name, symbol_name) = module_guard.get_import("multiply").unwrap();
        assert_eq!(module_name, "utils");
        assert_eq!(symbol_name, "multiply");
        
        // Clean up
        std::fs::remove_file("lib/utils.prism").unwrap();
        std::fs::remove_dir("lib").unwrap();
    }
} 