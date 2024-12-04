use std::env;
use std::error::Error;
use prism::{Interpreter, Parser, Lexer};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();
    
    // Get API key
    let api_key = env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY must be set in environment");

    // Initialize interpreter with medical functions
    let mut interpreter = Interpreter::new();
    interpreter.register_medical_functions(api_key);

    // Read and parse the demo file
    let source = include_str!("medical_diagnosis.prism");
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    // Execute the demo
    println!("Starting Medical Diagnosis Demo with Prism...\n");
    
    for stmt in statements {
        interpreter.eval_stmt(&stmt)?;
    }

    Ok(())
} 