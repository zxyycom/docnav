import { errorMessage } from "../../../../scripts/tools/errors.ts";
import { parseJsonValue } from "../../../../scripts/tools/json/value.ts";
import {
  isRecord,
  isStringArray,
  isUnknownArray
} from "../../../../scripts/tools/type-guards.ts";
import type { CommandRecord } from "../../smoke-harness.ts";

export type JsonRecord = Record<string, unknown>;

export function expectExit(record: CommandRecord, expected: number) {
  expect(record, record.exitCode === expected, `exit code is ${expected}`);
}

export function expectStdoutEmpty(record: CommandRecord) {
  expect(record, record.stdout.length === 0, "stdout is empty");
}

export function expectStderrEmpty(record: CommandRecord) {
  expect(record, record.stderr.length === 0, "stderr is empty");
}

export function expectStdoutIncludes(record: CommandRecord, value: string) {
  expectOutputIncludes(record, record.stdout, "stdout", value);
}

export function expectStderrIncludes(record: CommandRecord, value: string) {
  expectOutputIncludes(record, record.stderr, "stderr", value);
}

export function parseJson(record: CommandRecord): JsonRecord {
  return parseJsonObjectFromText(record, record.stdout, "stdout JSON", "stdout parses as JSON", "stdout JSON is an object");
}

export function expectIncludes(record: CommandRecord, values: unknown, value: string, summary: string) {
  expect(record, isUnknownArray(values) && values.includes(value), summary);
}

export function expect(record: CommandRecord, condition: unknown, summary: string) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

export function assertSetup(condition: unknown, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

export function expectJsonObject(record: CommandRecord, value: unknown, summary: string): JsonRecord {
  return expectTypedValue(record, value, summary, isJsonRecord, {});
}

export function expectObjectArray(record: CommandRecord, value: unknown, summary: string): JsonRecord[] {
  return expectTypedValue(
    record,
    value,
    summary,
    (item): item is JsonRecord[] => isUnknownArray(item) && item.every(isJsonRecord),
    []
  );
}

export function expectString(record: CommandRecord, value: unknown, summary: string): string {
  return expectTypedValue(record, value, summary, (item): item is string => typeof item === "string", "");
}

export function expectNumber(record: CommandRecord, value: unknown, summary: string): number {
  return expectTypedValue(record, value, summary, (item): item is number => typeof item === "number", 0);
}

export function expectStringArray(record: CommandRecord, value: unknown, summary: string): string[] {
  return expectTypedValue(record, value, summary, isStringArray, []);
}

export function parseJsonObjectFromText(
  record: CommandRecord,
  text: string,
  parseLabel: string,
  parseSummary: string,
  objectSummary: string
): JsonRecord {
  let value: unknown;
  try {
    value = parseJsonValue(text, parseLabel);
    record.assertions.push({ ok: true, summary: parseSummary });
  } catch (error: unknown) {
    record.assertions.push({
      ok: false,
      summary: `${parseSummary}: ${errorMessage(error)}`
    });
    throw error;
  }
  return expectJsonObject(record, value, objectSummary);
}

export function isJsonRecord(value: unknown): value is JsonRecord {
  return isRecord(value) && !isUnknownArray(value);
}

function expectOutputIncludes(record: CommandRecord, output: string, outputName: "stdout" | "stderr", value: string) {
  expect(record, output.includes(value), `${outputName} includes ${JSON.stringify(value)}`);
}

function expectTypedValue<T>(
  record: CommandRecord,
  value: unknown,
  summary: string,
  predicate: (value: unknown) => value is T,
  fallback: T
): T {
  if (predicate(value)) {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return fallback;
}
