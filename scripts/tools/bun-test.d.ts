declare module "bun:test" {
  type TestResult = unknown | Promise<unknown>;

  export function describe(name: string, fn: () => void): void;
  export function test(name: string, fn: () => TestResult): void;
  export function expect(value: unknown): {
    toBe(expected: unknown): void;
    toEqual(expected: unknown): void;
    toHaveLength(expected: number): void;
    toThrow(expected?: string): void;
  };
}
