# Prism API Reference

Complete API documentation for the Prism programming language.

## Core Types

### String

```prism
type string {
    // Methods
    length(): int
    to_upper(): string
    to_lower(): string
    trim(): string
    split(delimiter: string): List<string>
    contains(substring: string): bool
    replace(old: string, new: string): string
}
```

### Number Types

```prism
type int {
    // Methods
    to_string(): string
    to_float(): float
    abs(): int
    pow(exp: int): int
}

type float {
    // Methods
    to_string(): string
    to_int(): int
    round(): int
    ceil(): int
    floor(): int
}
```

### Collections

```prism
type List<T> {
    // Methods
    push(item: T): void
    pop(): T
    length(): int
    map<U>(fn: (T) -> U): List<U>
    filter(fn: (T) -> bool): List<T>
    reduce<U>(fn: (U, T) -> U, initial: U): U
}

type Map<K, V> {
    // Methods
    get(key: K): Option<V>
    set(key: K, value: V): void
    has(key: K): bool
    delete(key: K): bool
    keys(): List<K>
    values(): List<V>
}
```

## LLM Module

### Model Configuration

```prism
type ModelConfig {
    temperature: float
    max_tokens: int
    top_p: float
    frequency_penalty: float
    presence_penalty: float
}

type Model {
    // Methods
    new(config: ModelConfig): Model
    generate(prompt: string): Promise<string>
    stream(prompt: string): AsyncIterator<string>
}
```

### Conversation Management

```prism
type Message {
    role: string  // "user" | "assistant" | "system"
    content: string
}

type Conversation {
    // Methods
    new(): Conversation
    add_message(role: string, content: string): void
    get_messages(): List<Message>
    get_response(): Promise<string>
    clear(): void
}
```

## Medical Module

### Diagnosis Types

```prism
type Symptom {
    name: string
    severity: float
    duration: string
}

type Diagnosis {
    condition: string
    confidence: float
    recommendations: List<string>
}

type MedicalAnalyzer {
    // Methods
    validate_symptoms(symptoms: string): Promise<float>
    analyze(symptoms: string): Promise<Diagnosis>
    get_recommendations(diagnosis: Diagnosis): List<string>
}
```

## HTTP Module

### Request Types

```prism
type RequestConfig {
    headers: Map<string, string>
    params: Map<string, string>
    timeout: int
}

type Response {
    status: int
    headers: Map<string, string>
    text(): Promise<string>
    json(): Promise<json>
}

type HttpClient {
    // Methods
    get(url: string, config?: RequestConfig): Promise<Response>
    post(url: string, data: json, config?: RequestConfig): Promise<Response>
    put(url: string, data: json, config?: RequestConfig): Promise<Response>
    delete(url: string, config?: RequestConfig): Promise<Response>
}
```

## Error Handling

```prism
type Result<T, E> {
    // Methods
    is_ok(): bool
    is_err(): bool
    unwrap(): T
    unwrap_or(default: T): T
    map<U>(fn: (T) -> U): Result<U, E>
}

type Option<T> {
    // Methods
    is_some(): bool
    is_none(): bool
    unwrap(): T
    unwrap_or(default: T): T
    map<U>(fn: (T) -> U): Option<U>
}
```

## Testing

```prism
type TestSuite {
    // Methods
    test(name: string, fn: () -> void): void
    before_each(fn: () -> void): void
    after_each(fn: () -> void): void
    assert(condition: bool, message?: string): void
    assert_eq<T>(actual: T, expected: T, message?: string): void
}
```

For more information:
- [Language Guide](../guide/README.md)
- [Module System](../modules/README.md)
- [Standard Library](../stdlib/README.md) 