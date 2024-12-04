mod lexer;
mod parser;
mod ast;
mod types;
mod context;
mod confidence;
mod interpreter;
mod stdlib;
mod llm;

use std::io::{self, Write};
use std::process;
use colored::Colorize;
use miette::{Diagnostic, SourceSpan, IntoDiagnostic};
use thiserror::Error;

use crate::lexer::Lexer;
use crate::types::Value;
use crate::interpreter::Interpreter;
use crate::parser::Parser;

#[derive(Debug, Error, Diagnostic)]
#[error("Parse error: {message}")]
#[diagnostic(code(prism::parse_error))]
struct PrismParseError {
    #[source_code]
    src: String,
    #[label("Error occurred here")]
    span: SourceSpan,
    message: String,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Runtime error: {message}")]
#[diagnostic(code(prism::runtime_error))]
struct PrismRuntimeError {
    #[source_code]
    src: String,
    message: String,
}

fn main() -> miette::Result<()> {
    let mut interpreter = Interpreter::new();

    println!("{}", "Prism REPL".green().bold());
    println!("Type 'exit' to quit\n");

    loop {
        print!("{}", "prism> ".blue().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input == "exit" {
            process::exit(0);
        }

        match evaluate(&mut interpreter, input) {
            Ok(value) => println!("{}", value.to_string().yellow()),
            Err(err) => eprintln!("{}", err.to_string().red()),
        }
    }
}

fn evaluate(interpreter: &mut Interpreter, input: &str) -> miette::Result<Value> {
    let tokens = Lexer::new(input).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|err| PrismParseError {
        src: input.to_string(),
        span: (err.position..err.position + 1).into(),
        message: err.message,
    })?;

    let mut result = Value::Void;
    for stmt in statements {
        result = interpreter.eval_stmt(&stmt).map_err(|err| PrismRuntimeError {
            src: input.to_string(),
            message: err.to_string(),
        })?;
    }

    Ok(result)
} 