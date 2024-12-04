use std::collections::HashMap;
use crate::ast::{Stmt, Expr, MacroToken, MacroExpr, MacroOperator};
use crate::types::Confidence;

pub struct MacroExpander {
    definitions: HashMap<String, MacroDefinition>,
}

#[derive(Clone)]
struct MacroDefinition {
    parameters: Vec<String>,
    body: Vec<MacroToken>,
    confidence: Option<f64>,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn register_macro(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::MacroDefinition { name, parameters, body, confidence } => {
                let def = MacroDefinition {
                    parameters: parameters.clone(),
                    body: body.clone(),
                    confidence: *confidence,
                };
                self.definitions.insert(name.clone(), def);
                Ok(())
            },
            _ => Err("Not a macro definition".to_string()),
        }
    }

    pub fn expand_macro(&self, stmt: &Stmt) -> Result<Vec<Stmt>, String> {
        match stmt {
            Stmt::MacroInvoke { name, arguments, confidence } => {
                let def = self.definitions.get(name)
                    .ok_or_else(|| format!("Undefined macro: {}", name))?;
                
                if arguments.len() != def.parameters.len() {
                    return Err(format!(
                        "Macro {} expects {} arguments, got {}",
                        name,
                        def.parameters.len(),
                        arguments.len()
                    ));
                }
                
                // Create argument bindings
                let mut bindings = HashMap::new();
                for (param, arg) in def.parameters.iter().zip(arguments) {
                    bindings.insert(param.clone(), arg.clone());
                }
                
                // Expand macro body
                let mut expanded = Vec::new();
                for token in &def.body {
                    let stmts = self.expand_token(token, &bindings, *confidence)?;
                    expanded.extend(stmts);
                }
                
                Ok(expanded)
            },
            _ => Ok(vec![stmt.clone()]),
        }
    }

    fn expand_token(
        &self,
        token: &MacroToken,
        bindings: &HashMap<String, Expr>,
        confidence: Option<f64>,
    ) -> Result<Vec<Stmt>, String> {
        match token {
            MacroToken::Literal(s) => {
                Ok(vec![Stmt::Expression(Expr::String(s.clone()))])
            },
            
            MacroToken::Parameter(name) => {
                let expr = bindings.get(name)
                    .ok_or_else(|| format!("Undefined macro parameter: {}", name))?;
                Ok(vec![Stmt::Expression(expr.clone())])
            },
            
            MacroToken::Concat => {
                // String concatenation at compile time
                Ok(vec![])  // Handled in context
            },
            
            MacroToken::Stringify => {
                // Convert expression to string at compile time
                Ok(vec![])  // Handled in context
            },
            
            MacroToken::ConfidenceOf(param) => {
                let expr = bindings.get(param)
                    .ok_or_else(|| format!("Undefined macro parameter: {}", param))?;
                Ok(vec![Stmt::Expression(Expr::ConfidenceOf(Box::new(expr.clone())))])
            },
            
            MacroToken::ConfidenceFlow => {
                // Confidence flow operator
                Ok(vec![])  // Handled in context
            },
            
            MacroToken::If { condition, then_tokens, else_tokens } => {
                let cond_value = self.eval_macro_expr(condition, bindings)?;
                
                let tokens = if cond_value {
                    then_tokens
                } else if let Some(else_tokens) = else_tokens {
                    else_tokens
                } else {
                    return Ok(vec![]);
                };
                
                let mut expanded = Vec::new();
                for token in tokens {
                    let stmts = self.expand_token(token, bindings, confidence)?;
                    expanded.extend(stmts);
                }
                Ok(expanded)
            },
            
            MacroToken::Repeat { pattern, separator } => {
                let mut expanded = Vec::new();
                
                // Get list of items to repeat over (from bindings)
                let items = self.get_repeat_items(bindings)?;
                
                for (i, item) in items.iter().enumerate() {
                    // Create new bindings with current item
                    let mut new_bindings = bindings.clone();
                    new_bindings.insert("item".to_string(), item.clone());
                    new_bindings.insert("index".to_string(), Expr::Integer(i as i64));
                    
                    // Expand pattern
                    for token in pattern {
                        let stmts = self.expand_token(token, &new_bindings, confidence)?;
                        expanded.extend(stmts);
                    }
                    
                    // Add separator if needed
                    if i < items.len() - 1 {
                        if let Some(sep) = separator {
                            expanded.push(Stmt::Expression(Expr::String(sep.clone())));
                        }
                    }
                }
                
                Ok(expanded)
            },
        }
    }

    fn eval_macro_expr(
        &self,
        expr: &MacroExpr,
        bindings: &HashMap<String, Expr>,
    ) -> Result<bool, String> {
        match expr {
            MacroExpr::Boolean(b) => Ok(*b),
            
            MacroExpr::Parameter(name) => {
                let expr = bindings.get(name)
                    .ok_or_else(|| format!("Undefined macro parameter: {}", name))?;
                match expr {
                    Expr::Boolean(b) => Ok(*b),
                    _ => Err("Parameter is not a boolean".to_string()),
                }
            },
            
            MacroExpr::BinaryOp { op, left, right } => {
                let left_val = self.eval_macro_expr(left, bindings)?;
                let right_val = self.eval_macro_expr(right, bindings)?;
                
                match op {
                    MacroOperator::And => Ok(left_val && right_val),
                    MacroOperator::Or => Ok(left_val || right_val),
                    MacroOperator::Eq => Ok(left_val == right_val),
                    MacroOperator::NotEq => Ok(left_val != right_val),
                    _ => Err("Unsupported operator in macro condition".to_string()),
                }
            },
            
            MacroExpr::TypeCheck { value, type_name } => {
                let expr = match value.as_ref() {
                    MacroExpr::Parameter(name) => bindings.get(name)
                        .ok_or_else(|| format!("Undefined macro parameter: {}", name))?,
                    _ => return Err("Type check only works on parameters".to_string()),
                };
                
                Ok(expr.get_type().to_string() == *type_name)
            },
            
            MacroExpr::ConfidenceCheck { value, threshold } => {
                let expr = match value.as_ref() {
                    MacroExpr::Parameter(name) => bindings.get(name)
                        .ok_or_else(|| format!("Undefined macro parameter: {}", name))?,
                    _ => return Err("Confidence check only works on parameters".to_string()),
                };
                
                match expr.get_confidence() {
                    Some(conf) => Ok(conf > *threshold),
                    None => Ok(false),
                }
            },
            
            _ => Err("Invalid macro condition".to_string()),
        }
    }

    fn get_repeat_items(&self, bindings: &HashMap<String, Expr>) -> Result<Vec<Expr>, String> {
        // Look for array parameter in bindings
        if let Some(Expr::Array(items)) = bindings.get("items") {
            Ok(items.clone())
        } else {
            Err("No array found for repeat pattern".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_macro() {
        let mut expander = MacroExpander::new();
        
        // Define a simple macro
        let macro_def = Stmt::MacroDefinition {
            name: "print_twice".to_string(),
            parameters: vec!["msg".to_string()],
            body: vec![
                MacroToken::Parameter("msg".to_string()),
                MacroToken::Literal("\n".to_string()),
                MacroToken::Parameter("msg".to_string()),
            ],
            confidence: Some(0.9),
        };
        
        expander.register_macro(&macro_def).unwrap();
        
        // Test macro expansion
        let invoke = Stmt::MacroInvoke {
            name: "print_twice".to_string(),
            arguments: vec![Expr::String("Hello".to_string())],
            confidence: Some(0.95),
        };
        
        let expanded = expander.expand_macro(&invoke).unwrap();
        assert_eq!(expanded.len(), 3);
    }
    
    #[test]
    fn test_conditional_macro() {
        let mut expander = MacroExpander::new();
        
        // Define a macro with condition
        let macro_def = Stmt::MacroDefinition {
            name: "check_confidence".to_string(),
            parameters: vec!["value".to_string()],
            body: vec![
                MacroToken::If {
                    condition: Box::new(MacroExpr::ConfidenceCheck {
                        value: Box::new(MacroExpr::Parameter("value".to_string())),
                        threshold: 0.8,
                    }),
                    then_tokens: vec![MacroToken::Literal("High confidence".to_string())],
                    else_tokens: Some(vec![MacroToken::Literal("Low confidence".to_string())]),
                },
            ],
            confidence: None,
        };
        
        expander.register_macro(&macro_def).unwrap();
        
        // Test with high confidence value
        let high_conf = Stmt::MacroInvoke {
            name: "check_confidence".to_string(),
            arguments: vec![Expr::Float(0.9)],
            confidence: None,
        };
        
        let expanded = expander.expand_macro(&high_conf).unwrap();
        assert_eq!(expanded.len(), 1);
    }
} 