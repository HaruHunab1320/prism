# Prism Language

Prism is a domain-specific language designed for medical and healthcare applications with built-in LLM integration capabilities. It features confidence tracking, context management, and robust error handling.

## Current Status (v0.9)

The language is currently in late beta, with core features implemented and stabilizing for v1.0 release. See [ROADMAP.md](ROADMAP.md) for detailed timeline and [DEVELOPMENT.md](DEVELOPMENT.md) for current progress.

## Project Structure

```
prism/
├── compiler/           # Core compiler implementation
│   ├── src/           # Source code
│   ├── examples/      # Example programs
│   └── tests/         # Test suite
├── docs/              # Documentation
├── tools/             # Development tools
└── std/               # Standard library
    ├── core/          # Core functionality
    ├── utils/         # Utility functions
    ├── llm/           # LLM integration
    └── medical/       # Medical types and tools
```

## Features

### Implemented ✅
- Core language features (90%)
  - Variables, functions, and closures
  - Async/await support
  - Error handling with confidence tracking
  - Module system
  - Context management
  - Basic LLM integration

### In Progress 🚧
- LLM Integration (60%)
  - Basic provider interfaces
  - Completion API
  - Embeddings (WIP)
  - Fine-tuning (Planned)

- Medical Module (20%)
  - Basic health record types
  - Simple diagnosis tools
  - FHIR integration (Planned)

### Developer Tools
- REPL (80%)
- LSP Support (40%)
- Testing Framework (85%)
- Error Reporting (90%)

## Getting Started

1. **Installation**
```bash
git clone https://github.com/oneirocom/prism.git
cd prism
cargo build --release
```

2. **Running the REPL**
```bash
cargo run --bin prism-repl
```

3. **Running Tests**
```bash
cargo test
```

## Documentation

- [Getting Started Guide](docs/getting-started.md)
- [Language Reference](docs/reference/README.md)
- [Standard Library](docs/std/README.md)
- [Development Guide](DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)

## Examples

Check the `compiler/examples/` directory for sample programs demonstrating various language features:

- Basic syntax and control flow
- Async operations
- LLM integration
- Medical data processing
- Error handling with confidence

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to Prism.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Special thanks to all contributors and the medical informatics community for their valuable input and support.