use prism::value::ValueKind;
use prism::interpreter::Interpreter;
use prism::error::Result;

#[tokio::test]
pub async fn test_module_confidence() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        module test ~> 0.9 {
            export let x = 42;
        }
        import { x } from "test";
        x;
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_module_confidence_inheritance() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        module test ~> 0.9 {
            export fn get_value() -> number ~> 0.8 {
                return 42;
            }
        }
        import { get_value } from "test";
        get_value();
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_module_confidence_composition() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        module test ~> 0.9 {
            export fn get_value() -> number ~> 0.8 {
                return 42 ~> 0.7;
            }
        }
        import { get_value } from "test";
        get_value();
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_module_context() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        module test {
            export fn get_value() -> number {
                in context "test" {
                    return 42;
                }
            }
        }
        import { get_value } from "test";
        get_value();
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_module_confidence_and_context() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        module test ~> 0.9 {
            export fn get_value() -> number ~> 0.8 {
                in context "test" {
                    return 42 ~> 0.7;
                }
            }
        }
        import { get_value } from "test";
        get_value();
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
} 