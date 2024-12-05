# Prism TypeScript API Reference

## Core Classes

### `Prism`

The main class for interacting with the Prism runtime.

```typescript
class Prism {
    constructor(config?: PrismConfig);
    
    async eval<T>(code: string): Promise<PrismValue<T>>;
    getConfidence(value: PrismValue<unknown>): number;
    getContext(value: PrismValue<unknown>): string | undefined;
}
```

#### Configuration

```typescript
interface PrismConfig {
    apiKey?: string;          // API key for LLM features
    defaultConfidence?: number; // Default confidence level (0-1)
    defaultContext?: string;   // Default context name
}
```

#### Examples

```typescript
// Basic initialization
const prism = new Prism();

// With configuration
const prism = new Prism({
    apiKey: "your-api-key",
    defaultConfidence: 0.9,
    defaultContext: "Default"
});

// Evaluation
const result = await prism.eval<number>(`42 ~> 0.9`);
```

## Types

### `PrismValue<T>`

Represents a value with confidence and optional context.

```typescript
interface PrismValue<T = unknown> {
    value: T;              // The actual value
    confidence: number;    // Confidence level (0-1)
    context?: string;      // Optional context name
}
```

#### Examples

```typescript
// Typed value
const numResult: PrismValue<number> = {
    value: 42,
    confidence: 0.9,
    context: "Math"
};

// Complex type
interface UserData {
    name: string;
    age: number;
}

const userResult: PrismValue<UserData> = {
    value: { name: "Alice", age: 30 },
    confidence: 0.95,
    context: "UserProfile"
};
```

## Utility Functions

### `withConfidence<T>`

Create a value with confidence.

```typescript
function withConfidence<T>(value: T, confidence: number): PrismValue<T>;
```

#### Example

```typescript
const value = withConfidence(42, 0.9);
// { value: 42, confidence: 0.9 }
```

### `inContext<T>`

Execute code in a specific context.

```typescript
function inContext<T>(
    prism: Prism,
    context: string,
    code: string
): Promise<PrismValue<T>>;
```

#### Example

```typescript
const result = await inContext(prism, "Analysis", `
    process_data(input) ~> 0.8
`);
```

### `combineConfidence`

Combine multiple confidence values.

```typescript
function combineConfidence(confidences: number[]): number;
```

#### Example

```typescript
const combined = combineConfidence([0.9, 0.8, 0.95]);
// 0.684
```

### `isPrismValue<T>`

Type guard for PrismValue.

```typescript
function isPrismValue<T>(value: unknown): value is PrismValue<T>;
```

#### Example

```typescript
if (isPrismValue(result)) {
    console.log(result.confidence);
}
```

## Error Handling

### `PrismError`

Custom error class for Prism-specific errors.

```typescript
class PrismError extends Error {
    constructor(
        message: string,
        confidence?: number,
        context?: string
    );
    
    confidence?: number;
    context?: string;
}
```

#### Example

```typescript
try {
    await prism.eval(`invalid code`);
} catch (error) {
    if (error instanceof PrismError) {
        console.log(error.message);
        console.log(error.confidence);
        console.log(error.context);
    }
}
```

## Advanced Usage

### Type Inference

The `eval` method supports full TypeScript type inference:

```typescript
// Infer primitive types
const num = await prism.eval<number>(`42`);
const str = await prism.eval<string>(`"hello"`);

// Infer complex types
interface Data {
    count: number;
    items: string[];
}

const data = await prism.eval<Data>(`{
    count: 2,
    items: ["a", "b"]
} ~> 0.9`);
```

### Context Management

Contexts can be nested and combined:

```typescript
const result = await prism.eval<string>(`
    in context "Outer" {
        let x = "value" ~> 0.9;
        
        in context "Inner" {
            process_value(x) ~> 0.8
        }
    }
`);
```

### Confidence Flow

Confidence values propagate through operations:

```typescript
const result = await prism.eval<number>(`
    let a = 42 ~> 0.9;
    let b = 10 ~> 0.8;
    (a + b) ~> 0.95  // Combined confidence
`);
```

## Best Practices

1. Always specify types with `eval<T>`
2. Handle confidence appropriately
3. Use contexts to organize code
4. Catch and handle `PrismError`s
5. Use utility functions for common operations

## See Also

- [Getting Started Guide](./getting-started.md)
- [Examples](./examples.md)
- [Advanced Features](./advanced-features.md)
- [Migration Guide](./migration-guide.md) 