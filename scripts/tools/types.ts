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

export function isRecord(value: ExternalValue): value is Record<string, ExternalValue> {
  return typeof value === "object" && value !== null;
}

export function errorMessage(error: ExternalValue): string {
  return error instanceof Error ? error.message : String(error);
}

export function processFailure(error: ExternalValue): ProcessFailure {
  return error instanceof Error ? error as ProcessFailure : new Error(String(error));
}
