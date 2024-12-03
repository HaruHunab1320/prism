mod lexer;
mod parser;
mod ast;
mod types;
mod context;
mod confidence;
mod interpreter;
mod stdlib;
mod llm;

use std::error::Error;
use miette::{IntoDiagnostic, Result, Diagnostic};
use thiserror::Error;
use crate::lexer::Lexer;
use crate::parser::{Parser, ParseError};
use crate::interpreter::Interpreter;

#[derive(Error, Debug, Diagnostic)]
#[error("{message}")]
pub struct PrismError {
    message: String,
    #[label("here")]
    position: usize,
}

impl From<String> for PrismError {
    fn from(message: String) -> Self {
        Self {
            message,
            position: 0,
        }
    }
}

impl From<ParseError> for PrismError {
    fn from(err: ParseError) -> Self {
        Self {
            message: err.to_string(),
            position: err.location(),
        }
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("examples/basic.prism")
        .into_diagnostic()?;

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer);
    let mut interpreter = Interpreter::new();

    let mut last_value = None;

    while let Ok(stmt) = parser.parse_statement() {
        last_value = Some(interpreter.eval(&stmt)
            .map_err(PrismError::from)
            .into_diagnostic()?);
    }

    if let Some(value) = last_value {
        println!("Result: {:?}", value);
    }

    Ok(())
}

fn evaluate(interpreter: &mut Interpreter, input: &str) -> Result<interpreter::Value> {
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    println!("Tokens: {:?}", tokens);

    let mut parser = Parser::new(tokens.into_iter());
    let statements = parser.parse_program()
        .map_err(PrismError::from)
        .into_diagnostic()?;
    
    let mut last_value = interpreter::Value::Void;
    for stmt in statements {
        last_value = interpreter.eval(&stmt)
            .map_err(PrismError::from)
            .into_diagnostic()?;
    }
    Ok(last_value)
}

fn evaluate_file(interpreter: &mut Interpreter, path: std::path::PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(path).into_diagnostic()?;
    println!("Evaluating file content:\n{}", content);
    let result = evaluate(interpreter, &content)?;
    println!("Evaluation result:\n{:?}", result);
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
    println!("\nStandard Library Functions:");
    println!("  confidence.combine(x, y)   - Combine confidence values");
    println!("  confidence.max(x, y)       - Get maximum confidence");
    println!("  confidence.min(x, y)       - Get minimum confidence");
    println!("  pattern.match(text, pat)   - Pattern matching");
    println!("  pattern.semantic_match(text1, text2) - Semantic similarity");
    println!("  pattern.transform(text, transform)   - Text transformation");
    println!("  context.create(...)        - Create new context");
    println!("  verify.source(...)         - Verify against source");
} 