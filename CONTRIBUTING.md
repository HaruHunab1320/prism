# Contributing to Prism

We love your input! We want to make contributing to Prism as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## Development Setup

1. Install Rust toolchain (1.70.0 or later)
2. Install wasm-pack for WebAssembly builds
3. Clone the repository
4. Install dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Clone repository
git clone https://github.com/oneirocom/prism.git
cd prism

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

## Building and Testing

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

# Run WASM tests
wasm-pack test --chrome --features wasm
```

## Development Process

1. Fork the repo and create your branch from `main`
2. Make your changes
3. If you've added code that should be tested, add tests
4. If you've changed APIs, update the documentation
5. Ensure all tests pass (both native and WASM)
6. Create a pull request

## Pull Request Process

1. Update the README.md with details of changes to the interface
2. Update the DEVELOPMENT.md with details of new features or changes
3. The PR will be merged once you have the sign-off of two maintainers

## Any contributions you make will be under the MIT Software License

In short, when you submit code changes, your submissions are understood to be under the same [MIT License](http://choosealicense.com/licenses/mit/) that covers the project. Feel free to contact the maintainers if that's a concern.

## Report bugs using Github's [issue tracker](https://github.com/oneirocom/prism/issues)

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/oneirocom/prism/issues/new).

## Write bug reports with detail, background, and sample code

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## Use a Consistent Coding Style

* Use 4 spaces for indentation rather than tabs
* Run `cargo fmt` before committing
* Run `cargo clippy` to catch common mistakes
* Keep feature flags consistent between native and WASM builds

## License

By contributing, you agree that your contributions will be licensed under its MIT License. 