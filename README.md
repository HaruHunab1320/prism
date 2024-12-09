# Prism Language

A modern programming language designed for AI-first development with first-class support for confidence scores and context tracking.

## Current Status (v0.9)

✅ Core Language Features (90% complete):
- Lexer and Parser implementation
- Interpreter with async/await support
- Value system with confidence scores and context tracking
- Module system with proper scoping
- Basic type system
- Error handling (80%)
- Pattern matching (50%)

✅ Type System:
- Basic types (nil, boolean, number, string)
- Compound types (list, map)
- Function types with async support
- Module type with proper encapsulation
- Type inference foundation

✅ Standard Library Modules (75% complete):
- Core module (100%)
  - Basic operations
  - Type utilities
  - Assertions
- Utils module (100%)
  - File operations
  - JSON handling
  - Async utilities
- LLM module (70%)
  - Chat completion foundation
  - Model management structure
- Medical module (30%)
  - Basic structure
  - Diagnosis foundation

## Developer Experience (80% complete)
- Testing infrastructure (100%)
- Error reporting (90%)
- Module hot reloading (100%)
- REPL implementation (50%)
- Language server protocol (60%)

## Documentation (70% complete)
- Module system docs (100%)
- Standard library docs (90%)
- API reference (80%)
- Language specification (50%)
- Tutorials and examples (30%)

## Roadmap to v1.0

1. High Priority:
   - [ ] Complete LLM module implementation
   - [ ] Improve error handling system
   - [ ] Finish REPL implementation
   - [ ] Complete language specification

2. Medium Priority:
   - [ ] Complete Medical module
   - [ ] Enhance pattern matching
   - [ ] Add Data module
   - [ ] Add Network module

3. Lower Priority:
   - [ ] VS Code extension
   - [ ] Performance optimizations
   - [ ] Additional examples
   - [ ] Fuzzing tests

## Getting Started

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the REPL (coming soon)
cargo run
```

## Features

### Confidence Tracking
```rust
let x = 42 ~> 0.9;  // Value with 90% confidence
let y = "hello" ~> 0.95;  // Value with 95% confidence
```

### Context Awareness
```rust
let x = 42 @ "temperature reading";  // Value with context
let y = "hello" @ "user greeting";  // Value with context
```

### Async/Await
```rust
async fn get_data() -> Result<Value> {
    let response = await http.get("https://api.example.com/data");
    response.json()
}
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.