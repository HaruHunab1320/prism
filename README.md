# Prism Programming Language

A modern programming language designed for AI-first development, featuring confidence tracking, context awareness, and seamless LLM integration.

## Features

- Confidence tracking and propagation
- Context-aware computation
- Built-in LLM integration
- Pattern matching with confidence levels
- Async/await support
- Module system with confidence inheritance
- REPL environment for interactive development
- WebAssembly support

## Building

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Cargo package manager
- For WebAssembly builds: wasm-pack

### Native Build

```bash
# Build the library and CLI
cargo build --features native

# Run tests
cargo test --features native

# Start the REPL
cargo run --features native
```

### WebAssembly Build

```bash
# Build for wasm32 target
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm

# Build with wasm-pack (for npm package)
wasm-pack build --target web --features wasm
```

## Usage

### CLI/REPL

```bash
# Start the REPL
prism-cli

# Run a Prism file
prism-cli path/to/script.prism
```

### WebAssembly

```typescript
import { Prism } from 'prism-lang';

const prism = new Prism();

// Evaluate Prism code
const result = await prism.eval(`
    let x = 42 ~> 0.9;  // Value with 90% confidence
    x + 10
`);

console.log(result.value);      // 52
console.log(result.confidence); // 0.9
```

## Examples

### Basic Syntax

```prism
// Variable with confidence
let x = 42 ~> 0.9;

// Context-aware computation
in context "analysis" {
    let result = process_data(x) ~> 0.8;
    result
}

// Pattern matching with confidence
match value {
    x ~{0.8, 1.0} => "High confidence",
    x ~{0.5, 0.8} => "Medium confidence",
    _ => "Low confidence"
}
```

### Async Operations

```prism
async fn fetch_data() ~0.9 {
    let response = await http.get("https://api.example.com/data");
    response.json()
}

// Use in async context
let data = await fetch_data();
```

## Project Structure

```
prism/
├── compiler/         # Core language implementation
│   └── src/         # Source code
├── prism-ts/        # TypeScript/WebAssembly bindings
├── examples/        # Example Prism programs and comparisons
├── tests/          # Test suite
└── docs/           # Documentation
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on contributing to Prism.

## Development Status

See [DEVELOPMENT.md](DEVELOPMENT.md) for current development status and roadmap.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.