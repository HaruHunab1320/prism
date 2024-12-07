use std::env;
use std::fs;
use prism::interpreter::Interpreter;

#[tokio::main]
async fn main() {
    // Initialize environment
    prism::init();

    // Setup logging based on environment
    if env::var("PRISM_DEBUG").unwrap_or_default() == "true" {
        env_logger::init();
    }

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        std::process::exit(1);
    });

    let mut interpreter = Interpreter::new();
    match interpreter.evaluate(source).await {
        Ok(result) => println!("{:?}", result),
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
