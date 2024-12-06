mod parser_tests;
mod interpreter_tests;

#[tokio::test]
async fn run_all_tests() {
    // Parser Tests
    parser_tests::test_parse_function_declaration();
    parser_tests::test_parse_async_function();
    parser_tests::test_parse_let_declaration();
    parser_tests::test_parse_if_statement();
    parser_tests::test_arithmetic_precedence();
    parser_tests::test_logical_precedence();
    parser_tests::test_unary_expressions();
    parser_tests::test_call_expressions();
    
    // Interpreter Tests
    // Note: These tests are now run directly by the test runner
    // since they are marked with #[tokio::test]
    println!("All tests passed!");
}
