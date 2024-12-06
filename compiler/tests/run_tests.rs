use std::error::Error;

mod parser_tests;

#[tokio::test]
async fn run_all_tests() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Basic Declaration Tests
    parser_tests::test_parse_function_declaration();
    parser_tests::test_parse_async_function();
    parser_tests::test_parse_let_declaration();
    parser_tests::test_parse_if_statement();
    
    // Operator Precedence Tests
    parser_tests::test_arithmetic_precedence();
    parser_tests::test_logical_precedence();
    
    // Expression Tests
    parser_tests::test_unary_expressions();
    parser_tests::test_binary_expressions();
    parser_tests::test_call_expressions();
    
    // Error Recovery Tests
    parser_tests::test_missing_semicolon_recovery();
    parser_tests::test_missing_closing_brace_recovery();
    parser_tests::test_invalid_expression_recovery();
    
    Ok(())
}
