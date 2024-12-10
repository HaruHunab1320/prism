use std::error::Error;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::env;

pub async fn test_prism_features() -> Result<(), Box<dyn Error>> {
    println!("Testing Prism Language Features...\n");

    // Get API key from environment variable
    let api_key = env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());

    // Initialize interpreter with API key
    let mut interpreter = Interpreter::new(api_key);

    // Test 1: Context and Confidence Assignment
    println!("Test 1: Context and Confidence Assignment");
    let test1 = r#"
        context test_context {
            let base_confidence ~ 0.8;
            let result = base_confidence;
            result
        }
    "#;
    let result1 = interpreter.eval(test1.to_string()).await?;
    println!("Result: {:?}\n", result1);
    assert_eq!(result1.get_confidence(), Some(0.8));

    // Test 2: Confidence Flow
    println!("Test 2: Confidence Flow");
    let test2 = r#"
        let value = 100;
        let confidence = 0.9;
        let result = value ~> confidence;
        result
    "#;
    let result2 = interpreter.eval(test2.to_string()).await?;
    println!("Result: {:?}\n", result2);
    assert_eq!(result2.get_confidence(), Some(0.9));

    // Test 3: Verification
    println!("Test 3: Verification");
    let test3 = r#"
        verify against ["source1", "source2"] {
            let value = 42;
            value
        }
    "#;
    let result3 = interpreter.eval(test3.to_string()).await?;
    println!("Result: {:?}\n", result3);

    // Test 4: Uncertain If
    println!("Test 4: Uncertain If");
    let test4 = r#"
        let confidence = 0.85;
        uncertain if (confidence > 0.8) {
            "high confidence"
        } medium {
            "medium confidence"
        } low {
            "low confidence"
        }
    "#;
    let result4 = interpreter.eval(test4.to_string()).await?;
    println!("Result: {:?}\n", result4);

    // Test 5: Context-aware Expression
    println!("Test 5: Context-aware Expression");
    let test5 = r#"
        let value in "test_context" = 42;
        value
    "#;
    let result5 = interpreter.eval(test5.to_string()).await?;
    println!("Result: {:?}\n", result5);
    assert_eq!(result5.get_context(), Some("test_context".to_string()));

    // Test 6: Combined Features
    println!("Test 6: Combined Features");
    let test6 = r#"
        context analysis {
            let base_confidence ~ 0.8;
            
            verify against ["source1"] {
                uncertain if (base_confidence > 0.7) {
                    let result = 100 ~> base_confidence;
                    result
                } medium {
                    50
                } low {
                    0
                }
            }
        }
    "#;
    let result6 = interpreter.eval(test6.to_string()).await?;
    println!("Result: {:?}\n", result6);
    assert_eq!(result6.get_confidence(), Some(0.8));

    println!("All tests completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_all_features() {
        test_prism_features().await.unwrap();
    }
} 