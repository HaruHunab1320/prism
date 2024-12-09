mod parser_tests;
mod interpreter_tests;
mod module_tests;
mod integration_tests;

use prism::error::Result;

#[tokio::test]
async fn run_all_tests() -> Result<()> {
    // Parser tests
    parser_tests::test_parse_let_statement().await?;
    parser_tests::test_parse_function_declaration().await?;
    parser_tests::test_parse_if_statement().await?;
    parser_tests::test_parse_while_statement().await?;
    parser_tests::test_parse_expression().await?;

    // Module tests
    module_tests::test_basic_module().await?;
    module_tests::test_module_function_export().await?;
    module_tests::test_module_multiple_exports().await?;
    module_tests::test_module_confidence_propagation().await?;
    module_tests::test_module_not_found().await?;
    module_tests::test_module_symbol_not_found().await?;
    module_tests::test_module_context().await?;

    // Integration tests
    integration_tests::test_basic_execution().await?;
    integration_tests::test_variables().await?;
    integration_tests::test_scope().await?;
    integration_tests::test_conditionals().await?;
    integration_tests::test_loops().await?;
    integration_tests::test_functions().await?;
    integration_tests::test_error_handling().await?;

    Ok(())
}
