import { errorMessage } from "../errors.ts";
import { isRecord, isUnknownArray } from "../type-guards.ts";

export type JsonPrimitive = boolean | null | number | string;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
export type JsonObject = { [key: string]: JsonValue };

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
