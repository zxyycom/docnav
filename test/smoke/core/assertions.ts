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

export function expectCandidateWarning(record: CommandRecord, warning: unknown, expected: Record<string, unknown>) {
  const warningRecord = isRecord(warning) ? warning : {};
  const details = isRecord(warningRecord.details) ? warningRecord.details : {};
  expect(record, isRecord(warning), `candidate warning exists for ${String(expected.adapter_id)}`);
  for (const key of ["id", "reason", "effect", "details"]) {
    expect(record, Object.hasOwn(warningRecord, key), `candidate warning has ${key}`);
  }
  expect(record, warningRecord.id === "adapter_candidate_failure", "candidate warning id matches");
  expect(record, warningRecord.effect === "candidate_skipped", "candidate warning effect matches");
  expect(
    record,
    typeof warningRecord.reason === "string" && warningRecord.reason.length > 0,
    "candidate warning reason is nonempty"
  );
  for (const [key, value] of Object.entries(expected)) {
    expect(record, details[key] === value, `candidate warning details.${key} is ${String(value)}`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
