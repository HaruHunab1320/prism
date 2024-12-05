declare module "prism-wasm" {
  export class PrismRuntime {
    constructor();
    eval(code: string): Promise<any>;
    getConfidence(value: any): number;
    getContext(value: any): string | undefined;
  }

  export function create_value_with_confidence(
    value: any,
    confidence: number
  ): any;
  export function create_value_in_context(value: any, context: string): any;
}
