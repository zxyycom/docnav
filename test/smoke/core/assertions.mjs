import { expect } from "../../tools/cli-smoke/assertions.mjs";

export * from "../../tools/cli-smoke/assertions.mjs";

export function expectCandidateEvidence(record, candidate, expected) {
  expect(record, Boolean(candidate), `candidate evidence exists for ${expected.adapter_id}`);
  for (const key of ["adapter_id", "stage", "code", "reason", "details"]) {
    expect(record, Object.hasOwn(candidate, key), `candidate evidence has ${key}`);
  }
  for (const [key, value] of Object.entries(expected)) {
    expect(record, candidate[key] === value, `candidate ${key} is ${value}`);
  }
}

export function expectCandidateWarning(record, warning, expected) {
  expect(record, Boolean(warning), `candidate warning exists for ${expected.adapter_id}`);
  for (const key of ["id", "reason", "effect", "details"]) {
    expect(record, Object.hasOwn(warning, key), `candidate warning has ${key}`);
  }
  expect(record, warning.id === "adapter_candidate_failure", "candidate warning id matches");
  expect(record, warning.effect === "candidate_skipped", "candidate warning effect matches");
  expect(record, typeof warning.reason === "string" && warning.reason.length > 0, "candidate warning reason is nonempty");
  for (const [key, value] of Object.entries(expected)) {
    expect(record, warning.details?.[key] === value, `candidate warning details.${key} is ${value}`);
  }
}
