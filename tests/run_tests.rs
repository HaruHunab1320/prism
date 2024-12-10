mod parser_tests;
mod interpreter_tests;
mod module_tests;
mod integration_tests;

use prism::error::Result;

#[tokio::test]
pub async fn run_all_tests() -> Result<()> {
    // Parser tests (synchronous)
    parser_tests::test_parse_let_statement()?;
    parser_tests::test_parse_function()?;
    parser_tests::test_parse_if_statement()?;
    parser_tests::test_parse_while_statement()?;

    // Interpreter tests (asynchronous)
    interpreter_tests::test_basic_execution().await?;
    interpreter_tests::test_variables().await?;
    interpreter_tests::test_arithmetic().await?;
    interpreter_tests::test_functions().await?;
    interpreter_tests::test_conditionals().await?;
    interpreter_tests::test_loops().await?;
    interpreter_tests::test_scope().await?;
    interpreter_tests::test_error_handling().await?;

    // Module tests (asynchronous)
    module_tests::test_module_confidence().await?;
    module_tests::test_module_confidence_inheritance().await?;
    module_tests::test_module_confidence_composition().await?;
    module_tests::test_module_context().await?;
    module_tests::test_module_confidence_and_context().await?;

    // Integration tests (asynchronous)
    integration_tests::test_basic_execution().await?;
    integration_tests::test_variables().await?;
    integration_tests::test_scope().await?;
    integration_tests::test_conditionals().await?;
    integration_tests::test_loops().await?;
    integration_tests::test_functions().await?;
    integration_tests::test_error_handling().await?;

    Ok(())
} 