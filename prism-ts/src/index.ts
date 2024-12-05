import { PrismRuntime } from "prism-wasm";

export interface PrismValue<T = unknown> {
  value: T;
  confidence: number;
  context?: string;
}

export interface PrismConfig {
  apiKey?: string;
  defaultConfidence?: number;
  defaultContext?: string;
}

export class Prism {
  private runtime: PrismRuntime;
  private config: PrismConfig;

  constructor(config: PrismConfig = {}) {
    if (typeof window === "undefined") {
      throw new Error(
        "Prism requires a browser environment for WASM execution"
      );
    }

    // Initialize WASM module
    this.runtime = new PrismRuntime();
    this.config = {
      defaultConfidence: 1.0,
      ...config,
    };

    // Set API key if provided
    if (config.apiKey) {
      (window as any).PRISM_API_KEY = config.apiKey;
    }
  }

  /**
   * Evaluate Prism code and return the result with type safety
   */
  async eval<T>(code: string): Promise<PrismValue<T>> {
    const result = await this.runtime.eval(code);
    return result as PrismValue<T>;
  }

  /**
   * Get the confidence value of a Prism value
   */
  getConfidence(value: PrismValue<unknown>): number {
    return value.confidence;
  }

  /**
   * Get the context of a Prism value
   */
  getContext(value: PrismValue<unknown>): string | undefined {
    return value.context;
  }
}

/**
 * Create a value with confidence
 */
export function withConfidence<T>(value: T, confidence: number): PrismValue<T> {
  return {
    value,
    confidence,
  };
}

/**
 * Execute code in a specific context
 */
export async function inContext<T>(
  prism: Prism,
  context: string,
  code: string
): Promise<PrismValue<T>> {
  return await prism.eval<T>(`
        in context "${context}" {
            ${code}
        }
    `);
}

/**
 * Combine multiple confidence values
 */
export function combineConfidence(confidences: number[]): number {
  if (confidences.length === 0) return 1.0;
  return confidences.reduce((a, b) => a * b, 1.0);
}

// Type guards
export function isPrismValue<T>(value: unknown): value is PrismValue<T> {
  return (
    typeof value === "object" &&
    value !== null &&
    "value" in value &&
    "confidence" in value
  );
}

// Error types
export class PrismError extends Error {
  constructor(
    message: string,
    public confidence?: number,
    public context?: string
  ) {
    super(message);
    this.name = "PrismError";
  }
}

// Example usage:
/*
const prism = new Prism({ apiKey: 'your-api-key' });

// Basic evaluation
const result = await prism.eval<number>(`
    let x = 42 ~> 0.9;
    x + 10
`);
console.log(result); // { value: 52, confidence: 0.9 }

// Using contexts
const contextResult = await inContext(prism, "Analysis", `
    let data = analyze_data(input) ~> 0.8;
    return process_result(data) ~> 0.9;
`);

// Working with confidence
const combined = combineConfidence([0.9, 0.8, 0.95]);
console.log(combined); // 0.684

// Type-safe value creation
const safeValue = withConfidence({ count: 42 }, 0.9);
*/
