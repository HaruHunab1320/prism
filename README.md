# Project Prism

A programming language designed specifically for LLM interactions and probabilistic computing.

## Overview

Prism is a domain-specific language that brings probabilistic computing and LLM interactions into the core language design. It features:

- First-class support for confidence values and uncertainty
- Built-in context management
- Native LLM integration with Google's Gemini API
- Advanced pattern matching with semantic understanding
- Source verification and hallucination prevention

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- Google Cloud account with Gemini API access

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/prism.git
cd prism
```

2. Set up environment:
```bash
# Copy example environment file
cp .env.example .env

# Edit .env and add your Google API key
# Get your API key from: https://console.cloud.google.com/
nano .env
```

3. Build the project:
```bash
cd compiler
cargo build
```

4. Run an example:
```bash
# Basic example
cargo run examples/basic.prism

# LLM integration example
cargo run examples/llm.prism
```

## Language Features

### Confidence Values
```prism
conf x = 0.8  // Declare a confidence value
x ~> 0.7      // Confidence flow
```

### Uncertain Control Flow
```prism
uncertain if (condition ~> 0.7) {
    // High confidence path
} medium {
    // Medium confidence path
} low {
    // Low confidence fallback
}
```

### Context Management
```prism
in context Medical {
    conf diagnosis = 0.8
    verify against sources ["pubmed"] {
        conf treatment = 0.7
    }
}
```

### LLM Integration

#### Semantic Pattern Matching
```prism
conf match = pattern.semantic_match(
    "The patient shows signs of elevated blood pressure",
    "The individual's blood pressure readings are above normal"
)
```

#### Source Verification
```prism
verify against sources ["medical_guidelines"] {
    conf verification = verify.source("medical_guidelines", diagnosis)
}
```

#### Pattern Transformation
```prism
let transformed = pattern.transform(
    "The patient shows signs of elevated blood pressure",
    "Convert to medical terminology"
)
```

## Configuration

The following environment variables can be set in `.env`:

- `GOOGLE_API_KEY` (required): Your Google API key
- `GEMINI_MODEL` (optional): Override default model (default: "gemini-pro")
- `GEMINI_TIMEOUT_SECS` (optional): API timeout in seconds (default: 30)
- `GEMINI_MAX_RETRIES` (optional): Maximum retry attempts (default: 3)

## Development Status

This project is in early development. Current status:

- [x] Core language design
- [x] Basic compiler infrastructure
- [x] Lexer implementation
- [x] Parser implementation
- [x] Interpreter
- [x] Standard library
- [x] LLM integrations
- [ ] IDE support

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 