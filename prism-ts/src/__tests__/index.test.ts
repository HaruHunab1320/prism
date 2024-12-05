import {
  Prism,
  withConfidence,
  inContext,
  combineConfidence,
  PrismError,
} from "../index";

describe("Prism TypeScript Integration", () => {
  let prism: Prism;

  beforeEach(() => {
    prism = new Prism({ apiKey: "test-key" });
  });

  test("basic evaluation", async () => {
    const result = await prism.eval<number>(`
            let x = 42 ~> 0.9;
            x + 10
        `);

    expect(result.value).toBe(52);
    expect(result.confidence).toBe(0.9);
  });

  test("context management", async () => {
    const result = await inContext(
      prism,
      "testing",
      `
            let x = "test" ~> 0.8;
            x
        `
    );

    expect(result.context).toBe("testing");
    expect(result.confidence).toBe(0.8);
  });

  test("confidence helpers", () => {
    const value = withConfidence(42, 0.9);
    expect(value.value).toBe(42);
    expect(value.confidence).toBe(0.9);

    const combined = combineConfidence([0.9, 0.8, 0.95]);
    expect(combined).toBeCloseTo(0.684, 3);
  });

  test("error handling", async () => {
    await expect(
      prism.eval(`
            let x = 1 / 0;
        `)
    ).rejects.toThrow(PrismError);
  });

  test("type safety", async () => {
    interface User {
      name: string;
      age: number;
    }

    const result = await prism.eval<User>(`
            {
                name: "Alice" ~> 0.9,
                age: 30 ~> 0.95
            }
        `);

    expect(result.value.name).toBe("Alice");
    expect(result.value.age).toBe(30);
    expect(result.confidence).toBeCloseTo(0.855, 3); // 0.9 * 0.95
  });

  test("async operations", async () => {
    const result = await prism.eval<string>(`
            let response = await llm.analyze("Test");
            response.text
        `);

    expect(typeof result.value).toBe("string");
    expect(result.confidence).toBeGreaterThan(0);
  });
});
