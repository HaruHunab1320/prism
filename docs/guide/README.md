# Prism Language Guide

Welcome to the Prism Language Guide. This guide will help you learn and master Prism programming language.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Concepts](#basic-concepts)
3. [Advanced Features](#advanced-features)
4. [Working with AI/LLM](#working-with-ai-llm)

## Getting Started

### Installation

```bash
cargo install prism-lang
```

### Your First Prism Program

```prism
fn main() {
    println("Hello, Prism!")
}
```

## Basic Concepts

### Variables and Types

```prism
let name: string = "Prism"
let age: int = 1
let is_awesome: bool = true
```

### Functions

```prism
fn add(a: int, b: int) -> int {
    return a + b
}
```

### Control Flow

```prism
if condition {
    // code
} else {
    // code
}

for item in items {
    // code
}
```

## Advanced Features

### Async Programming

```prism
async fn fetch_data() -> Result<string> {
    let response = await http.get("https://api.example.com")
    return response.text()
}
```

### Error Handling

```prism
try {
    let result = risky_operation()
} catch error {
    println("Error: {}", error)
}
```

## Working with AI/LLM

### Basic LLM Integration

```prism
import llm from "std/llm"

async fn generate_text(prompt: string) -> string {
    return await llm.generate(prompt)
}
```

### Advanced AI Features

```prism
import { Classifier } from "std/llm/classification"

let classifier = Classifier.new()
let result = await classifier.classify("Sample text")
```

For more detailed information about specific topics, please refer to:
- [Module System Guide](../modules/README.md)
- [Standard Library Reference](../stdlib/README.md)
- [Language Specification](../SPEC.md) 