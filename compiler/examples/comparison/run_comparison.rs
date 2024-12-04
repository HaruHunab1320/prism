use prism::{
    error::RuntimeError,
    interpreter::Interpreter,
    stdlib::{
        core::create_core_module,
        utils::create_utils_module,
        llm::create_llm_module,
        medical::create_medical_module,
    },
    ast::{Stmt, Expr},
    types::Value,
};
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    // Create modules
    let core_module = create_core_module();
    let utils_module = create_utils_module();
    let llm_module = create_llm_module();
    let medical_module = create_medical_module();

    // Create interpreter and register modules
    let mut interpreter = Interpreter::new();
    interpreter.register_module(&["core"], core_module)?;
    interpreter.register_module(&["utils"], utils_module)?;
    interpreter.register_module(&["llm"], llm_module)?;
    interpreter.register_module(&["medical"], medical_module)?;

    // Run comparison
    let start = Instant::now();
    let result = run_comparison(&mut interpreter).await?;
    let duration = start.elapsed();

    println!("Comparison completed in {:?}", duration);
    println!("Result: {}", result);

    Ok(())
}

async fn run_comparison(interpreter: &mut Interpreter) -> Result<Value, RuntimeError> {
    // For this example, we'll create a simpler version of the test
    let statements = vec![
        Arc::new(Stmt::Let {
            name: "symptom".to_string(),
            initializer: Arc::new(Expr::String("fever".to_string())),
        }),
        Arc::new(Stmt::Let {
            name: "disease".to_string(),
            initializer: Arc::new(Expr::String("flu".to_string())),
        }),
        Arc::new(Stmt::Let {
            name: "validated".to_string(),
            initializer: Arc::new(Expr::Call {
                function: Arc::new(Expr::Binary {
                    left: Arc::new(Expr::Identifier("medical".to_string())),
                    operator: "_".to_string(),
                    right: Arc::new(Expr::Identifier("validate_symptom".to_string())),
                }),
                arguments: vec![Arc::new(Expr::Identifier("symptom".to_string()))],
            }),
        }),
        Arc::new(Stmt::Let {
            name: "match_score".to_string(),
            initializer: Arc::new(Expr::Call {
                function: Arc::new(Expr::Binary {
                    left: Arc::new(Expr::Identifier("medical".to_string())),
                    operator: "_".to_string(),
                    right: Arc::new(Expr::Identifier("semantic_match".to_string())),
                }),
                arguments: vec![
                    Arc::new(Expr::Identifier("symptom".to_string())),
                    Arc::new(Expr::Identifier("disease".to_string())),
                ],
            }),
        }),
        Arc::new(Stmt::Expression(Arc::new(Expr::Object(vec![
            ("symptom".to_string(), Arc::new(Expr::Identifier("symptom".to_string()))),
            ("validated".to_string(), Arc::new(Expr::Identifier("validated".to_string()))),
            ("match_score".to_string(), Arc::new(Expr::Identifier("match_score".to_string()))),
        ])))),
    ];

    interpreter.interpret(statements).await
} 