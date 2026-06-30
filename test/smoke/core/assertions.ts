import { expect } from "../../tools/cli-smoke/assertions.ts";
import type { CommandRecord } from "../../tools/smoke-harness.ts";

export * from "../../tools/cli-smoke/assertions.ts";

export function expectFormatCandidate(record: CommandRecord, candidate: unknown, expected: Record<string, unknown>) {
  const candidateRecord = isRecord(candidate) ? candidate : {};
  expect(record, isRecord(candidate), `format candidate exists for ${String(expected.adapter_id)}`);
  const keys = Object.keys(candidateRecord).sort().join(",");
  expect(record, keys === "adapter_id,reason,stage", "format candidate only has adapter_id, reason, and stage");
  for (const key of ["adapter_id", "stage", "reason"]) {
    expect(record, Object.hasOwn(candidateRecord, key), `format candidate has ${key}`);
  }
  for (const [key, value] of Object.entries(expected)) {
    expect(record, candidateRecord[key] === value, `candidate ${key} is ${String(value)}`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
