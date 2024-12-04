use std::collections::{HashMap, HashSet};
use crate::ast::{Expr, Stmt, Type, Operator};
use crate::types::Confidence;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeVar {
    Concrete(Type),
    Generic(String),
    Inferred(usize),
}

#[derive(Debug)]
pub struct TypeEnvironment {
    vars: HashMap<String, TypeVar>,
    constraints: Vec<(TypeVar, TypeVar, Option<f64>)>,  // Type equality constraints with confidence
    next_var: usize,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            constraints: Vec::new(),
            next_var: 0,
        }
    }

    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar::Inferred(self.next_var);
        self.next_var += 1;
        var
    }

    pub fn add_constraint(&mut self, t1: TypeVar, t2: TypeVar, confidence: Option<f64>) {
        self.constraints.push((t1, t2, confidence));
    }

    pub fn unify(&mut self) -> Result<HashMap<usize, Type>, String> {
        let mut substitutions = HashMap::new();

        for (t1, t2, confidence) in self.constraints.clone() {
            self.unify_types(&t1, &t2, &mut substitutions, confidence)?;
        }

        Ok(substitutions)
    }

    fn unify_types(
        &self,
        t1: &TypeVar,
        t2: &TypeVar,
        subst: &mut HashMap<usize, Type>,
        confidence: Option<f64>,
    ) -> Result<(), String> {
        match (t1, t2) {
            (TypeVar::Concrete(ty1), TypeVar::Concrete(ty2)) => {
                if ty1 == ty2 {
                    Ok(())
                } else {
                    Err(format!("Type mismatch: {:?} != {:?}", ty1, ty2))
                }
            },
            
            (TypeVar::Inferred(id), ty) | (ty, TypeVar::Inferred(id)) => {
                if let TypeVar::Inferred(other_id) = ty {
                    if id == other_id {
                        return Ok(());
                    }
                }
                
                if self.occurs_check(*id, ty) {
                    return Err("Recursive type".to_string());
                }
                
                match ty {
                    TypeVar::Concrete(concrete_ty) => {
                        subst.insert(*id, concrete_ty.clone());
                        Ok(())
                    },
                    TypeVar::Generic(name) => {
                        subst.insert(*id, Type::Generic(name.clone()));
                        Ok(())
                    },
                    TypeVar::Inferred(other_id) => {
                        subst.insert(*id, Type::Inferred(*other_id));
                        Ok(())
                    },
                }
            },
            
            (TypeVar::Generic(name1), TypeVar::Generic(name2)) => {
                if name1 == name2 {
                    Ok(())
                } else {
                    Err(format!("Generic type mismatch: {} != {}", name1, name2))
                }
            },
            
            _ => Err("Cannot unify types".to_string()),
        }
    }

    fn occurs_check(&self, id: usize, ty: &TypeVar) -> bool {
        match ty {
            TypeVar::Inferred(other_id) => id == *other_id,
            TypeVar::Concrete(_) => false,
            TypeVar::Generic(_) => false,
        }
    }
}

pub struct TypeInferer {
    env: TypeEnvironment,
}

impl TypeInferer {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
        }
    }

    pub fn infer_expr(&mut self, expr: &Expr) -> Result<(TypeVar, Option<f64>), String> {
        match expr {
            Expr::Integer(_) => Ok((TypeVar::Concrete(Type::Integer), Some(1.0))),
            Expr::Float(_) => Ok((TypeVar::Concrete(Type::Float), Some(1.0))),
            Expr::String(_) => Ok((TypeVar::Concrete(Type::String), Some(1.0))),
            Expr::Boolean(_) => Ok((TypeVar::Concrete(Type::Boolean), Some(1.0))),
            
            Expr::Identifier(name) => {
                if let Some(ty) = self.env.vars.get(name) {
                    Ok((ty.clone(), Some(1.0)))
                } else {
                    let var = self.env.fresh_var();
                    self.env.vars.insert(name.clone(), var.clone());
                    Ok((var, Some(1.0)))
                }
            },
            
            Expr::BinaryOp { op, left, right } => {
                let (left_ty, left_conf) = self.infer_expr(left)?;
                let (right_ty, right_conf) = self.infer_expr(right)?;
                
                let result_ty = match op {
                    Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => {
                        self.env.add_constraint(left_ty.clone(), right_ty.clone(), None);
                        left_ty.clone()
                    },
                    Operator::Eq | Operator::NotEq => {
                        self.env.add_constraint(left_ty.clone(), right_ty.clone(), None);
                        TypeVar::Concrete(Type::Boolean)
                    },
                    Operator::ConfFlow => {
                        // Confidence flow preserves type but adjusts confidence
                        left_ty.clone()
                    },
                    _ => return Err("Unsupported binary operator".to_string()),
                };
                
                // Combine confidences
                let combined_conf = match (left_conf, right_conf) {
                    (Some(c1), Some(c2)) => Some(c1 * c2),
                    _ => None,
                };
                
                Ok((result_ty, combined_conf))
            },
            
            Expr::Call { function, arguments } => {
                let (func_ty, func_conf) = self.infer_expr(function)?;
                let mut arg_types = Vec::new();
                let mut arg_confs = Vec::new();
                
                for arg in arguments {
                    let (arg_ty, arg_conf) = self.infer_expr(arg)?;
                    arg_types.push(arg_ty);
                    arg_confs.push(arg_conf);
                }
                
                let return_ty = self.env.fresh_var();
                
                // Function type constraint
                let func_type = Type::Function {
                    parameters: arg_types.iter().map(|ty| match ty {
                        TypeVar::Concrete(t) => t.clone(),
                        _ => Type::Any,
                    }).collect(),
                    return_type: Box::new(match &return_ty {
                        TypeVar::Concrete(t) => t.clone(),
                        _ => Type::Any,
                    }),
                };
                
                self.env.add_constraint(func_ty, TypeVar::Concrete(func_type), None);
                
                // Combine all confidences
                let combined_conf = std::iter::once(func_conf)
                    .chain(arg_confs)
                    .flatten()
                    .fold(Some(1.0), |acc, conf| {
                        acc.map(|a| a * conf)
                    });
                
                Ok((return_ty, combined_conf))
            },
            
            _ => Err("Unsupported expression for type inference".to_string()),
        }
    }

    pub fn infer_stmt(&mut self, stmt: &Stmt) -> Result<Option<f64>, String> {
        match stmt {
            Stmt::Let { name, value, type_annotation, confidence } => {
                let (inferred_ty, inferred_conf) = self.infer_expr(value)?;
                
                if let Some(annotated_ty) = type_annotation {
                    self.env.add_constraint(
                        inferred_ty.clone(),
                        TypeVar::Concrete(annotated_ty.clone()),
                        *confidence,
                    );
                }
                
                self.env.vars.insert(name.clone(), inferred_ty);
                
                // Combine let confidence with inferred confidence
                Ok(match (confidence, inferred_conf) {
                    (Some(c1), Some(c2)) => Some(c1 * c2),
                    (Some(c), None) | (None, Some(c)) => Some(c),
                    (None, None) => None,
                })
            },
            
            Stmt::Function { name, parameters, return_type, body, confidence } => {
                // Create new scope for function
                let old_vars = self.env.vars.clone();
                self.env.vars.clear();
                
                // Add parameters to environment
                for (param_name, param_type) in parameters {
                    self.env.vars.insert(
                        param_name.clone(),
                        TypeVar::Concrete(param_type.clone()),
                    );
                }
                
                // Infer body
                let mut body_conf = Some(1.0);
                for stmt in body {
                    if let Some(stmt_conf) = self.infer_stmt(stmt)? {
                        body_conf = body_conf.map(|c| c * stmt_conf);
                    }
                }
                
                // Restore outer scope
                self.env.vars = old_vars;
                
                // Combine function confidence with body confidence
                Ok(match (confidence, body_conf) {
                    (Some(c1), Some(c2)) => Some(c1 * c2),
                    (Some(c), None) | (None, Some(c)) => Some(c),
                    (None, None) => None,
                })
            },
            
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_inference() {
        let mut inferer = TypeInferer::new();
        
        // Test integer literal
        let expr = Expr::Integer(42);
        let (ty, conf) = inferer.infer_expr(&expr).unwrap();
        assert_eq!(ty, TypeVar::Concrete(Type::Integer));
        assert_eq!(conf, Some(1.0));
        
        // Test binary operation
        let expr = Expr::BinaryOp {
            op: Operator::Add,
            left: Box::new(Expr::Integer(1)),
            right: Box::new(Expr::Integer(2)),
        };
        let (ty, conf) = inferer.infer_expr(&expr).unwrap();
        assert_eq!(ty, TypeVar::Concrete(Type::Integer));
        assert_eq!(conf, Some(1.0));
    }
    
    #[test]
    fn test_function_inference() {
        let mut inferer = TypeInferer::new();
        
        let stmt = Stmt::Function {
            name: "add".to_string(),
            parameters: vec![
                ("x".to_string(), Type::Integer),
                ("y".to_string(), Type::Integer),
            ],
            return_type: Some(Type::Integer),
            body: vec![
                Stmt::Return(Expr::BinaryOp {
                    op: Operator::Add,
                    left: Box::new(Expr::Identifier("x".to_string())),
                    right: Box::new(Expr::Identifier("y".to_string())),
                }),
            ],
            confidence: Some(0.9),
        };
        
        let conf = inferer.infer_stmt(&stmt).unwrap();
        assert_eq!(conf, Some(0.9));
    }
} 