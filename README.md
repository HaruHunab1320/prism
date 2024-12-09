# Prism Language

A modern programming language designed for AI-first development with first-class support for confidence scores and context tracking.

## Current Status (v0.9)

### ✅ Working Features

The following features are fully implemented and tested:

1. Basic Language Features:
   - Variable declarations with `let`
   - Numeric literals and arithmetic operations
   - String literals
   - Expression evaluation
   - Proper operator precedence
   - Grouped expressions with parentheses

2. Arithmetic Operations:
   - Addition (+)
   - Subtraction (-)
   - Multiplication (*)
   - Division (/)
   - Complex nested expressions

3. REPL Features:
   - Interactive expression evaluation
   - Variable state management
   - Expression result display
   - Debug output for evaluation steps

Example:
```prism
// Variable declarations
let x = 42;
let y = 10;

// Arithmetic operations
let sum = x + y;      // 52
let diff = x - y;     // 32
let prod = x * y;     // 420
let quot = x / y;     // 4.2

// String handling
let message = "Hello, Prism!";

// Complex expressions
let result = (x + y) * (x - y);  // 1664
```

## Core Language Features (90% complete):
- ✅ Lexer and Parser implementation
- ✅ Basic Interpreter functionality
- ✅ Value system
- ⏳ Module system with proper scoping (In Progress)
- ⏳ Basic type system (In Progress)
- ⏳ Error handling (80%)
- ⏳ Pattern matching (50%)

✅ Type System:
- ✅ Basic types (nil, boolean, number, string)
- ⏳ Compound types (list, map) (In Progress)
- ⏳ Function types with async support (Planned)
- ⏳ Module type with proper encapsulation (Planned)
- ⏳ Type inference foundation (Planned)

✅ Standard Library Modules (In Development):
- Core module (50%)
  - ✅ Basic operations
  - ⏳ Type utilities (Planned)
  - ⏳ Assertions (Planned)
- Utils module (25%)
  - ⏳ File operations (Planned)
  - ⏳ JSON handling (Planned)
  - ⏳ Async utilities (Planned)
- LLM module (10%)
  - ⏳ Chat completion foundation (Planned)
  - ⏳ Model management structure (Planned)
- Medical module (5%)
  - ⏳ Basic structure (Planned)
  - ⏳ Diagnosis foundation (Planned)

## Developer Experience
- ✅ Basic REPL functionality (100%)
- ✅ Basic error reporting (100%)
- ⏳ Testing infrastructure (In Progress)
- ⏳ Module hot reloading (Planned)
- ⏳ Language server protocol (Planned)

## Documentation (40% complete)
- ✅ Basic language features (100%)
- ✅ REPL usage (100%)
- ⏳ Module system docs (Planned)
- ⏳ Standard library docs (Planned)
- ⏳ API reference (In Progress)
- ⏳ Language specification (In Progress)
- ⏳ Tutorials and examples (In Progress)

## Roadmap to v1.0

1. High Priority:
   - [x] Basic arithmetic operations
   - [x] Variable declarations
   - [x] REPL implementation
   - [ ] Complete error handling system
   - [ ] Basic type system implementation
   - [ ] Function definitions and calls

2. Medium Priority:
   - [ ] Module system implementation
   - [ ] Standard library core module
   - [ ] Control flow statements
   - [ ] Pattern matching

3. Lower Priority:
   - [ ] Advanced type system features
   - [ ] Standard library expansion
   - [ ] Performance optimizations
   - [ ] Development tools

## Getting Started

```bash
# Build the project
cargo build

# Run the REPL
cargo run

# Run a Prism script
cargo run examples/demo.prism
```

## Example Programs

### Basic Arithmetic
```prism
// Variable declarations and arithmetic
let x = 42;
let y = 10;

// Basic operations
let sum = x + y;      // 52
let diff = x - y;     // 32
let prod = x * y;     // 420
let quot = x / y;     // 4.2

// Complex expressions
let result = (x + y) * (x - y);  // 1664
```

### String Handling
```prism
let message = "Hello, Prism!";
```

More examples coming soon!

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.