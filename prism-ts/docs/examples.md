# Prism TypeScript Examples

This document provides practical examples of using Prism in TypeScript applications.

## Basic Examples

### Simple Calculations with Confidence

```typescript
import { Prism } from 'prism-lang';

const prism = new Prism();

async function calculateWithConfidence() {
    const result = await prism.eval<number>(`
        let x = 42 ~> 0.9;  // 90% confident
        let y = 10 ~> 0.8;  // 80% confident
        x + y               // Confidence combines automatically
    `);

    console.log(result.value);       // 52
    console.log(result.confidence);  // ~0.72 (0.9 * 0.8)
}
```

### String Processing

```typescript
async function processText() {
    const result = await prism.eval<string>(`
        let text = "Hello, World!" ~> 1.0;
        let processed = text.to_upper() ~> 0.95;
        processed + "!"
    `);

    console.log(result.value);      // "HELLO, WORLD!!"
    console.log(result.confidence); // 0.95
}
```

## Working with LLMs

### Basic LLM Integration

```typescript
const prism = new Prism({
    apiKey: process.env.PRISM_API_KEY
});

async function generateText() {
    const result = await prism.eval<string>(`
        let prompt = "Explain quantum computing" ~> 1.0;
        let response = await llm.generate(prompt) ~> 0.9;
        response
    `);

    console.log(result.value);       // Generated explanation
    console.log(result.confidence);  // Confidence in the response
}
```

### Context-Aware LLM Processing

```typescript
async function analyzeWithContext() {
    const result = await prism.eval<{
        sentiment: string;
        confidence: number;
    }>(`
        in context "SentimentAnalysis" {
            let text = "I love this product!" ~> 1.0;
            let analysis = await llm.analyze_sentiment(text) ~> 0.9;
            
            in context "Validation" {
                verify analysis.score > 0.5 {
                    return {
                        sentiment: "positive",
                        confidence: analysis.score
                    } ~> 0.95;
                }
            }
        }
    `);

    console.log(result.value);
    console.log(result.context);     // "SentimentAnalysis"
}
```

## Data Processing

### Type-Safe Data Transformation

```typescript
interface InputData {
    name: string;
    age: number;
}

interface OutputData {
    displayName: string;
    isAdult: boolean;
    confidence: number;
}

async function transformData(input: InputData) {
    const result = await prism.eval<OutputData>(`
        let data = ${JSON.stringify(input)} ~> 0.9;
        
        {
            displayName: data.name.to_upper(),
            isAdult: data.age >= 18,
            confidence: 0.95
        } ~> 0.9
    `);

    return result.value;
}
```

### Batch Processing with Confidence

```typescript
async function processBatch(items: string[]) {
    const results = await Promise.all(
        items.map(item => 
            prism.eval<{
                processed: string;
                score: number;
            }>(`
                let item = ${JSON.stringify(item)} ~> 1.0;
                let processed = process_item(item) ~> 0.9;
                let score = calculate_score(processed) ~> 0.8;
                
                {
                    processed: processed,
                    score: score
                } ~> min(0.9, score)
            `)
        )
    );

    return results.filter(r => r.confidence > 0.7);
}
```

## Error Handling

### Graceful Error Recovery

```typescript
import { PrismError } from 'prism-lang';

async function safeEvaluation() {
    try {
        const result = await prism.eval<number>(`
            let x = risky_operation() ~> 0.5;
            if confidence(x) < 0.7 {
                throw error("Low confidence", confidence: 0.5);
            }
            x * 2
        `);
        
        return result.value;
    } catch (error) {
        if (error instanceof PrismError) {
            console.log(`Error confidence: ${error.confidence}`);
            console.log(`Error context: ${error.context}`);
            return null;
        }
        throw error;
    }
}
```

### Confidence-Based Flow Control

```typescript
async function processWithConfidence(data: unknown) {
    const result = await prism.eval<{
        success: boolean;
        message: string;
    }>(`
        let confidence_threshold = 0.8;
        let processed = process_data(${JSON.stringify(data)}) ~> 0.9;
        
        if confidence(processed) >= confidence_threshold {
            return {
                success: true,
                message: "Processing successful"
            } ~> confidence(processed);
        } else {
            return {
                success: false,
                message: "Low confidence in processing"
            } ~> confidence(processed);
        }
    `);

    if (!result.value.success) {
        console.log(`Low confidence: ${result.confidence}`);
    }
    
    return result.value;
}
```

## Advanced Patterns

### Context-Based Configuration

```typescript
async function configureByContext() {
    return await prism.eval<{
        config: Record<string, unknown>;
        environment: string;
    }>(`
        in context "Development" {
            {
                config: {
                    debug: true,
                    apiUrl: "http://localhost:3000"
                },
                environment: "development"
            } ~> 1.0
        } shift to "Production" {
            {
                config: {
                    debug: false,
                    apiUrl: "https://api.production.com"
                },
                environment: "production"
            } ~> 0.99
        }
    `);
}
```

### Confidence Aggregation

```typescript
import { combineConfidence } from 'prism-lang';

async function aggregateResults() {
    const results = await Promise.all([
        prism.eval<number>('process1() ~> 0.9'),
        prism.eval<number>('process2() ~> 0.8'),
        prism.eval<number>('process3() ~> 0.95')
    ]);

    const confidences = results.map(r => r.confidence);
    const totalConfidence = combineConfidence(confidences);

    return {
        values: results.map(r => r.value),
        confidence: totalConfidence
    };
}
```

## See Also

- [API Reference](./api-reference.md)
- [Getting Started Guide](./getting-started.md)
- [Advanced Features](./advanced-features.md) 