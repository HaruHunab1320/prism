# Prism Programming Language

Prism is a modern programming language designed for AI-first development, featuring seamless LLM integration, confidence tracking, and context management.

## Features

### Core Language Features (Implemented ✅)
- Variables and scoping
- Functions and closures
- Async/await support
- Control flow (if/while/for)
- Error handling
- Expression evaluation
- Block scoping
- Basic operators (arithmetic, logical, comparison)
- String operations
- Testing infrastructure

### Module System (In Progress 🚧)
- Module structure and interfaces defined
- Basic module registration
- Standard library module placeholders
- TypeScript/WASM integration foundation

### AI Features (In Progress 🚧)
- Confidence tracking (partial)
- Context management (planned)
- LLM integration (planned)
- Pattern matching (planned)
- Medical domain support (planned)

### Type System (Planned 📋)
- Static type checking
- Type inference
- Generics
- Traits and interfaces
- Custom types and structs

### Standard Library (Planned 📋)
- Data structures
- String manipulation
- Math functions
- File I/O
- Network operations
- JSON handling

## Getting Started

### Prerequisites
- Rust toolchain (latest stable version)
- Cargo package manager

### Installation
```bash
# Clone the repository
git clone https://github.com/oneirocom/prism.git

# Build the project
cd prism
cargo build --release
```

### Basic Usage
```rust
// Hello World
let message = "Hello, World!";
print(message);

// Function declaration
fn add(a, b) {
    return a + b;
}

// Async function
async fn fetch_data() {
    // Async operations here
}

// Error handling
fn safe_divide(a, b) {
    if (b == 0) {
        return null;
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