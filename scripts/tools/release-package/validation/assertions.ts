export function assertEqualLists(
  actual: string[],
  expected: string[],
  message: string,
): void {
  assert(
    actual.length === expected.length &&
      actual.every((value, index) => value === expected[index]),
    message,
  );
}

export function assertNonEmptyString(
  value: unknown,
  label: string,
): asserts value is string {
  assert(
    typeof value === "string" && value.length > 0,
    `${label} must be a string`,
  );
}

export function assertPositiveInteger(
  value: unknown,
  label: string,
): asserts value is number {
  assert(
    typeof value === "number" && Number.isInteger(value) && value > 0,
    `${label} must be a positive integer`,
  );
}

export function assert(
  condition: unknown,
  message: string,
): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}
