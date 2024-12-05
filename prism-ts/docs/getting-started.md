# Getting Started with Prism in TypeScript

## Installation

```bash
npm install prism-lang
# or
yarn add prism-lang
```

## Basic Usage

```typescript
import { Prism, PrismValue } from 'prism-lang';

// Initialize Prism
const prism = new Prism({
    apiKey: 'your-api-key',  // Optional: for LLM features
    defaultConfidence: 0.9    // Optional: default confidence level
});

// Basic evaluation
async function example() {
    const result = await prism.eval<number>(`
        let x = 42 ~> 0.9;  // Value with 90% confidence
        x + 10
    `);
    
    console.log(result);
    // Output: { value: 52, confidence: 0.9 }
}

// Using contexts
async function contextExample() {
    const result = await prism.eval<string>(`
        in context "Analysis" {
            let data = "example" ~> 0.8;
            process_data(data)
        }
    `);
    
    console.log(result.context);  // "Analysis"
    console.log(result.confidence);  // 0.8
}
```

## Key Features

### 1. Confidence Flow

Prism's confidence system is fully accessible in TypeScript:

```typescript
import { withConfidence, combineConfidence } from 'prism-lang';

// Create values with confidence
const value = withConfidence(42, 0.9);

// Combine confidence values
const combined = combineConfidence([0.9, 0.8, 0.95]);
console.log(combined);  // 0.684
```

### 2. Context Management

Work with Prism's context system:

```typescript
import { inContext } from 'prism-lang';

async function contextExample() {
    const result = await inContext(prism, "Analysis", `
        // This code runs in the Analysis context
        process_data(input) ~> 0.8
    `);
}
```

### 3. Type Safety

Prism provides full TypeScript type safety:

```typescript
interface UserData {
    name: string;
    score: number;
}

async function processUser() {
    const result = await prism.eval<UserData>(`
        {
            name: "Alice",
            score: 95
        } ~> 0.9
    `);
    
    // TypeScript knows the shape of result.value
    console.log(result.value.name);    // Type-safe
    console.log(result.value.score);   // Type-safe
}
```

### 4. Error Handling

```typescript
import { PrismError } from 'prism-lang';

try {
    const result = await prism.eval(`
        // Invalid code
        let x = ;
    `);
} catch (error) {
    if (error instanceof PrismError) {
        console.log(error.confidence);  // Confidence of the error
        console.log(error.context);     // Context where error occurred
    }
}
```

## Best Practices

1. **Type Everything**
   ```typescript
   // Good
   interface AnalysisResult {
       score: number;
       category: string;
   }
   const result = await prism.eval<AnalysisResult>(code);

   // Avoid
   const result = await prism.eval(code);  // unknown type
   ```

2. **Handle Confidence**
   ```typescript
   // Check confidence before using results
   const result = await prism.eval<number>(code);
   if (result.confidence > 0.8) {
       // High confidence path
   } else {
       // Low confidence path
   }
   ```

3. **Use Contexts Appropriately**
   ```typescript
   // Group related operations in contexts
   const result = await inContext(prism, "UserAnalysis", `
       let user = get_user() ~> 0.9;
       let preferences = analyze_preferences(user) ~> 0.8;
       generate_recommendations(preferences) ~> 0.7
   `);
   ```

## Next Steps

- Check out the [API Reference](./api-reference.md)
- See [Examples](./examples.md)
- Learn about [Advanced Features](./advanced-features.md)