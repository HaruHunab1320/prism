mod lexer;
mod parser;
mod ast;
mod types;
mod context;
mod confidence;

use miette::{Result, IntoDiagnostic};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::confidence::ConfidenceEngine;
use crate::context::ContextManager;
use std::fs;
use std::path::PathBuf;

struct Interpreter {
    confidence_engine: ConfidenceEngine,
    context_manager: ContextManager,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            confidence_engine: ConfidenceEngine::new(0.1), // Default decay rate
            context_manager: ContextManager::new(),
        }
    }

    fn evaluate(&mut self, input: &str) -> Result<String> {
        println!("Tokenizing input: {}", input);
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        println!("Tokens: {:?}", tokens);

        let mut parser = Parser::new(tokens.into_iter());
        
        match parser.parse_program() {
            Ok(statements) => {
                let mut output = Vec::new();
                for stmt in statements {
                    output.push(format!("{}", stmt));
                }
                Ok(output.join("\n"))
            }
            Err(e) => Ok(format!("Parse error: {:?}", e)),
        }
    }

    fn evaluate_file(&mut self, path: PathBuf) -> Result<()> {
        let content = fs::read_to_string(path).into_diagnostic()?;
        println!("Evaluating file content:\n{}", content);
        let result = self.evaluate(&content)?;
        println!("Evaluation result:\n{}", result);
        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut interpreter = Interpreter::new();

    if args.len() > 1 {
        // Run file mode
        let path = PathBuf::from(&args[1]);
        interpreter.evaluate_file(path)?;
    } else {
        // REPL mode
        println!("Prism Programming Language v0.1.0");
        println!("Type 'help' for available commands or ':q' to quit");

        let mut input = String::new();
        loop {
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout()).into_diagnostic()?;
            
            input.clear();
            std::io::stdin().read_line(&mut input).into_diagnostic()?;
            let input = input.trim();

            if input == ":q" {
                break;
            }

            match input {
                "help" => print_help(),
                _ => {
                    match interpreter.evaluate(input) {
                        Ok(result) => println!("{}", result),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  help     - Show this help message");
    println!("  :q       - Quit the REPL");
    println!("\nExample expressions:");
    println!("  conf x = 0.8");
    println!("  uncertain if (x ~> 0.7) {{ ... }}");
    println!("  in context Medical {{ ... }}");
    println!("  verify against sources {{ ... }}");
} 