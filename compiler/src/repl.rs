use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use crate::interpreter::Interpreter;
use crate::error::{Result, PrismError};
use crate::value::Value;

pub struct Repl {
    interpreter: Interpreter,
    editor: DefaultEditor,
}

impl Repl {
    pub fn new() -> Result<Self> {
        let mut editor = DefaultEditor::new().map_err(|e| PrismError::RuntimeError(e.to_string()))?;
        editor.load_history("history.txt").ok(); // Don't fail if no history

        Ok(Self {
            interpreter: Interpreter::new(),
            editor,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Prism REPL v0.9.0");
        println!("Type 'help' for more information or 'exit' to quit");

        loop {
            match self.editor.readline("prism> ") {
                Ok(line) => {
                    self.editor.add_history_entry(&line).map_err(|e| PrismError::RuntimeError(e.to_string()))?;
                    
                    match line.trim() {
                        "exit" | "quit" => break,
                        "help" => self.print_help(),
                        input => {
                            match self.eval(input).await {
                                Ok(value) => println!("{:?}", value),
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }

        self.editor.save_history("history.txt").map_err(|e| PrismError::RuntimeError(e.to_string()))?;
        Ok(())
    }

    async fn eval(&mut self, input: &str) -> Result<Value> {
        self.interpreter.evaluate(input.to_string()).await
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  help     - Show this help message");
        println!("  exit     - Exit the REPL");
        println!("  quit     - Exit the REPL");
        println!("\nExample expressions:");
        println!("  42                     - Number literal");
        println!("  \"Hello\"                - String literal");
        println!("  [1, 2, 3]              - List literal");
        println!("  {{\"key\": \"value\"}}       - Map literal");
        println!("  let x = 42 ~> 0.9      - Variable with confidence");
        println!("  let y = \"hi\" @ \"greeting\" - Variable with context");
        println!("\nFor more information, visit: https://github.com/oneirocom/prism");
    }
} 