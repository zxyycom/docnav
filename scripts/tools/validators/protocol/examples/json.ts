import { assert } from "../../assertions.ts";
import { isRecord } from "../../../foundation/src/type-guards.ts";

export function jsonObject(value: unknown, label: string): Record<string, unknown> {
  assert(isRecord(value), `${label} must be an object`);
  return value;
}

export function jsonArray(value: unknown, label: string): unknown[] {
  assert(Array.isArray(value), `${label} must be an array`);
  return value;
}

export function codePointLength(value: string): number {
  return [...value].length;
}
