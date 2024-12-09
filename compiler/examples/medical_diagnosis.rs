use prism::Interpreter;
use prism::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut interpreter = Interpreter::new();
    
    let source = r#"
        module medical ~> 0.9 {
            export fn diagnose(symptoms: string) -> string ~> 0.8 {
                // Simplified diagnosis logic
                if symptoms.contains("fever") {
                    return "Possible flu" ~> 0.7;
                } else {
                    return "Unknown condition" ~> 0.5;
                }
            }
        }

        import { diagnose } from "medical";
        let result = diagnose("high fever and cough");
        print(result);
    "#;

    let result = interpreter.evaluate(source.to_string()).await?;
    println!("Result: {:?}", result);
    Ok(())
}
