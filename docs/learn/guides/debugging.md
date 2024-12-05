# Debugging Guide for Prism

This guide covers debugging techniques and tools for Prism applications, with a focus on confidence-aware and context-sensitive debugging.

## Table of Contents
- [Basic Debugging](#basic-debugging)
- [Confidence Debugging](#confidence-debugging)
- [Context Debugging](#context-debugging)
- [LLM Integration Debugging](#llm-integration-debugging)
- [Common Issues](#common-issues)

## Basic Debugging

### Using the Debug Macro

```prism
import debug from "std/debug"

fn example() {
    let x = 42
    debug!("Value of x: {}", x)  // Prints with source location
}
```

### Debug Mode Execution

Run your Prism program in debug mode:
```bash
prism run --debug program.prism
```

### Interactive Debugging

Use the Prism REPL for interactive debugging:
```bash
prism repl --load program.prism
```

## Confidence Debugging

### Confidence Flow Tracing

```prism
fn process_data(input: string) ~0.9 {
    debug_confidence!("Input confidence", input)
    
    let validated = validate(input) ~> 0.8
    debug_confidence!("Validation confidence", validated)
    
    return validated
}
```

### Confidence Breakpoints

Set breakpoints that trigger based on confidence levels:

```prism
#[confidence_break(0.5)]  // Break if confidence drops below 0.5
fn risky_operation() ~0.7 {
    // Your code here
}
```

## Context Debugging

### Context Stack Tracing

```prism
in context DataProcessing {
    debug_context!()  // Prints current context stack
    
    process_data()
}
```

### Context Transition Logging

```prism
#[trace_context_transitions]
fn process_workflow() {
    in context Validation {
        // Operations
    } shift to Processing {
        // More operations
    }
}
```

## LLM Integration Debugging

### LLM Request Tracing

```prism
import { debug_llm } from "std/debug/llm"

async fn generate_text() {
    debug_llm!("Starting generation")
    let response = await llm.generate("prompt")
    debug_llm!("Generation complete", response)
}
```

### Mock LLM Responses

```prism
#[test]
fn test_llm_integration() {
    mock_llm!({
        input: "What is 2+2?",
        output: "4",
        confidence: 0.95
    })
    
    let result = llm.generate("What is 2+2?")
    assert_eq(result, "4")
}
```

## Common Issues

### 1. Low Confidence Issues

```prism
// Problem
let result = process() ~> 0.3  // Low confidence

// Solution
let result = process() ~> 0.3
if confidence(result) < 0.5 {
    debug!("Low confidence detected: {}", confidence(result))
    // Handle low confidence
}
```

### 2. Context Leaks

```prism
// Problem
in context A {
    let x = 1
}
print(x)  // Context leak

// Solution
in context A {
    let x = 1
    debug_context!("Variable x scope", x)
} // x is properly scoped
```

### 3. LLM Timeouts

```prism
// Problem
let response = await llm.generate("prompt")  // Might timeout

// Solution
try {
    let response = await timeout(
        llm.generate("prompt"),
        5000  // 5 second timeout
    )
} catch TimeoutError {
    debug!("LLM request timed out")
}
```

## Debugging Tools

### 1. Prism Inspector

Visual debugging tool for Prism applications:
```bash
prism inspect program.prism
```

### 2. Confidence Profiler

Profile confidence flow in your application:
```bash
prism profile --confidence program.prism
```

### 3. Context Analyzer

Analyze context transitions and potential issues:
```bash
prism analyze --context program.prism
```

## Best Practices

1. **Use Structured Logging**
   ```prism
   debug!({
       level: "error",
       context: "validation",
       confidence: 0.8,
       message: "Validation failed"
   })
   ```

2. **Set Confidence Breakpoints Strategically**
   ```prism
   #[confidence_break(
       threshold: 0.5,
       message: "Low confidence detected",
       action: "pause"
   )]
   ```

3. **Monitor Context Transitions**
   ```prism
   #[trace_context(
       transitions: true,
       stack: true,
       variables: true
   )]
   ```

## Integration with Development Tools

### VS Code Extension

Install the Prism VS Code extension for integrated debugging:
```bash
code --install-extension prism-debug
```

### Command Line Tools

```bash
# Start debugging session
prism debug

# Attach debugger to running process
prism debug --attach <pid>

# Start debugging with specific configuration
prism debug --config debug.json
```

For more information:
- [Language Specification](../reference/SPEC.md)
- [VS Code Integration](../learn/guides/vscode.md)
- [Testing Guide](../learn/guides/testing.md) 