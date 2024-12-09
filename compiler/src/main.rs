use std::env;
use std::fs;
use prism::interpreter::Interpreter;
use prism::repl::Repl;
use prism::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment
    prism::init();

    // Setup logging based on environment
    if env::var("PRISM_DEBUG").unwrap_or_default() == "true" {
        env_logger::init();
    }

    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        // No arguments - start REPL
        1 => {
            let mut repl = Repl::new()?;
            repl.run().await?;
        }
        // One argument - execute file
        2 => {
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
        // Invalid usage
        _ => {
            eprintln!("Usage: prism [source_file]");
            eprintln!("  Run without arguments to start REPL");
            std::process::exit(1);
        }
    }

    Ok(())
}
