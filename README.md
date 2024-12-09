# Prism Language

A modern programming language designed for AI-first development with first-class support for confidence scores and context tracking.

## Current Status (v0.9)

### ✅ Working Features

The following features are fully implemented and tested:

1. Variable System:
   - Variable declarations with `let`
   - Integer and floating-point numbers
   - String literals
   - Nil values
   - Variable shadowing
   - Variable scope and state management

2. Arithmetic Operations:
   - Integer arithmetic (+, -, *, /)
   - Floating-point arithmetic
   - Mixed integer/float operations
   - Proper operator precedence
   - Parenthesized expressions

3. Expression System:
   - Complex nested expressions
   - Multi-term calculations
   - Proper order of operations
   - Expression result handling
   - Expression statement evaluation

4. REPL Features:
   - Interactive expression evaluation
   - Variable state persistence
   - Multi-line input support
   - Expression result display
   - Detailed debug output

Example:
```prism
// Variable declarations with different types
let x = 42;              // Integer
let y = 3.14;           // Float
let name = "Prism";     // String
let empty = nil;        // Nil value

// Arithmetic and expressions
let sum = x + 10;           // Basic arithmetic
let complex = (x + y) * 2;  // Mixed types
let nested = ((x + 10) * (y + 5)) / 2;  // Complex nesting

// Variable reuse and shadowing
let a = 10;
let b = a + 5;     // Variable reuse
let a = a + 1;     // Variable shadowing
```

## Implementation Status

### Core Language Features:
- ✅ Lexer and Parser (100%)
  - Token recognition
  - Expression parsing
  - Statement parsing
  - Error reporting
- ✅ Basic Interpreter (100%)
  - Expression evaluation
  - Statement execution
  - Variable management
  - Debug output
- ✅ Value System (100%)
  - Numbers (Integer/Float)
  - Strings
  - Nil values
  - Value type checking
- ⏳ Control Flow (Planned)
  - If/Else statements
  - Loops
  - Function definitions
- ⏳ Error Handling (In Progress)
  - Basic error types
  - Error propagation
  - Recovery mechanisms

✅ Type System:
- ✅ Basic Types
  - Numbers (Integer/Float)
  - Strings
  - Nil
- ⏳ Advanced Types (Planned)
  - Boolean
  - Arrays
  - Objects
  - Functions
  - Custom types

## Developer Experience
- ✅ REPL Environment (100%)
  - Interactive execution
  - State management
  - Debug output
- ✅ Script Execution (100%)
  - File loading
  - Script parsing
  - Full execution
- ⏳ Development Tools (Planned)
  - Debugger
  - Code formatter
  - Language server

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
   - [x] Variable system
   - [x] Basic arithmetic
   - [x] Expression evaluation
   - [ ] Control flow statements
   - [ ] Function definitions
   - [ ] Error handling improvements

2. Medium Priority:
   - [ ] Boolean operations
   - [ ] Array support
   - [ ] Object system
   - [ ] Standard library

3. Lower Priority:
   - [ ] Type system expansion
   - [ ] Module system
   - [ ] Development tools
   - [ ] Performance optimizations

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