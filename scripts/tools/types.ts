export type JsonPrimitive = boolean | null | number | string;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
export type JsonObject = { [key: string]: JsonValue };
export type StringMap = Record<string, string>;

export interface ProcessFailure extends Error {
  code?: number | string | null;
  signal?: NodeJS.Signals | null;
  status?: number | null;
  stderr?: Buffer | string;
  stdout?: Buffer | string;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function isNonArrayRecord(value: unknown): value is Record<string, unknown> {
  return isRecord(value) && !Array.isArray(value);
}

export function isUnknownArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

export function isStringArray(value: unknown): value is string[] {
  return isUnknownArray(value) && value.every((item) => typeof item === "string");
}

export function isJsonValue(value: unknown): value is JsonValue {
  if (
    value === null ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    typeof value === "string"
  ) {
    return true;
  }
  if (isUnknownArray(value)) {
    return value.every(isJsonValue);
  }
  if (isRecord(value)) {
    return Object.values(value).every(isJsonValue);
  }
  return false;
}

export function parseJsonValue(source: string, label = "JSON"): JsonValue {
  let parsed: unknown;
  try {
    parsed = JSON.parse(source);
  } catch (error: unknown) {
    throw new Error(`${label} parse failed: ${errorMessage(error)}`, { cause: error });
  }
  if (!isJsonValue(parsed)) {
    throw new Error(`${label} must contain a JSON value`);
  }
  return parsed;
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function parsePositiveInteger(value: number | string, label: string): number {
  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isInteger(parsed) || parsed <= 0 || String(parsed) !== String(value)) {
    throw new Error(`${label} must be a positive integer: ${value}`);
  }

  return parsed;
}

export function processFailure(error: unknown): ProcessFailure {
  return error instanceof Error ? error as ProcessFailure : new Error(String(error));
}
