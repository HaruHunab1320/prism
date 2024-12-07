# Prism Programming Language

Prism is a modern programming language designed for AI-first development, featuring seamless LLM integration, confidence tracking, and context management.

## Features

### Core Language Features (Implemented ✅)
- Variables and scoping
- Functions and closures
- Async/await support
- Control flow (if/while/for)
- Error handling with confidence tracking
- Expression evaluation
- Block scoping
- Basic operators (arithmetic, logical, comparison)
- String operations
- Testing infrastructure
- Method chaining
- Property access
- Circular dependency detection

### Module System (Implemented ✅)
- Module structure and interfaces
- Module registration and loading
- Import/export system with confidence propagation
- Module dependency resolution
- Circular dependency detection
- Standard library module structure
- Module caching
- TypeScript/WASM integration foundation

### AI Features (In Progress 🚧)
- Confidence tracking ✅
- Context management ✅
- Confidence propagation ✅
- LLM integration (in progress)
- Pattern matching (planned)
- Medical domain support (planned)

### Type System (Planned 📋)
- Static type checking
- Type inference
- Generics
- Traits and interfaces
- Custom types and structs

### Standard Library (In Progress 🚧)
- Core module ✅
  - Basic operations
  - Type utilities
  - Assertions
- LLM module (in progress)
  - Chat completion
  - Embeddings
  - Model management
- Medical module (planned)
  - Diagnosis helpers
  - Health record types
  - FHIR integration
- Utils module ✅
  - String manipulation
  - Math functions
  - Basic I/O

## Getting Started

### Prerequisites
- Rust toolchain (latest stable version)
- Cargo package manager

### Installation
bash
# Clone the repository
git clone https://github.com/oneirocom/prism.git

# Build the project
cd prism
cargo build --release
```

### Basic Usage
```rust
// Hello World
let message = "Hello, World!" ~> 0.9;
print(message);

// Function with confidence
fn add(a, b) ~> 0.95 {
    return a + b;
}

// Module definition and import
module math ~> 0.9 {
    export fn multiply(a, b) {
        return a * b;
    }
}

import { multiply } from "math" ~> 0.8;
let result = multiply(2, 3);  // Combined confidence: 0.9 * 0.8

// Async function with context
async fn fetch_data() ~> 0.8 {
    in context "medical" {
        let response = await llm.analyze("patient symptoms");
        return response ~> 0.9;
    }
}

// Error handling with confidence
fn safe_divide(a, b) ~> 0.95 {
    if (b == 0) {
        throw error("Division by zero", confidence: 0.99);
    }
    return a / b;
}
```

### Running Tests
```bash
cargo test
```

## Project Structure
```
prism/
├── compiler/           # Core compiler implementation
│   ├── src/
│   │   ├── lexer.rs   # Lexical analysis
│   │   ├── parser.rs  # Syntax parsing
│   │   ├── ast.rs     # Abstract Syntax Tree
│   │   ├── module.rs  # Module system
│   │   ├── value.rs   # Value representation
│   │   ├── stdlib/    # Standard library modules
│   │   └── ...
│   └── tests/         # Test suite
├── examples/          # Example programs
├── prism-ts/         # TypeScript/WASM integration
└── docs/             # Documentation
```

## Contributing
Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## Development Status
See [DEVELOPMENT.md](DEVELOPMENT.md) for current development status and progress tracking.

## Roadmap
See [ROADMAP.md](ROADMAP.md) for planned features and development timeline.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```