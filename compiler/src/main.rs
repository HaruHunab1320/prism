use std::error::Error;
use std::fs;
use std::path::PathBuf;

use prism::Interpreter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: prism <file>");
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    let source = fs::read_to_string(&path)?;

    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    match interpreter.eval(source).await {
        Ok(result) => {
            println!("{:#?}", result);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
