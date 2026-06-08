import { envelopeKeys } from "./config.mjs";

export function expectExit(record, expected) {
  expect(record, record.exitCode === expected, `exit code is ${expected}`);
}

export function expectStdoutEmpty(record) {
  expect(record, record.stdout.length === 0, "stdout is empty");
}

export function expectStderrEmpty(record) {
  expect(record, record.stderr.length === 0, "stderr is empty");
}

export function expectStdoutIncludes(record, text) {
  expect(record, record.stdout.includes(text), `stdout includes ${JSON.stringify(text)}`);
}

export function expectStderrIncludes(record, text) {
  expect(record, record.stderr.includes(text), `stderr includes ${JSON.stringify(text)}`);
}

export function parseJson(record) {
  try {
    const value = JSON.parse(record.stdout);
    record.assertions.push({ ok: true, summary: "stdout parses as JSON" });
    return value;
  } catch (error) {
    record.assertions.push({
      ok: false,
      summary: `stdout parses as JSON: ${error.message}`
    });
    throw error;
  }
}

export function expectNoProtocolEnvelope(record, value) {
  const found = findProtocolEnvelopeKeys(value);
  for (const key of envelopeKeys) {
    const leakedPaths = found.filter((item) => item.key === key).map((item) => item.path);
    const location = leakedPaths.length > 0 ? ` at ${leakedPaths.join(", ")}` : "";
    expect(record, leakedPaths.length === 0, `readable JSON omits ${key}${location}`);
  }
}

export function expectProtocolSuccess(record, value, operation) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof value.request_id === "string" && value.request_id.length > 0, "request_id is nonempty");
  expect(record, value.operation === operation, `operation is ${operation}`);
  expect(record, value.ok === true, "ok is true");
  expect(record, Object.hasOwn(value, "result"), "protocol success has result");
  expectNoWarningsField(record, value, "protocol-json stdout");
}

export function expectProtocolFailure(record, value, operation, code) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof value.request_id === "string" && value.request_id.length > 0, "request_id is nonempty");
  expect(record, value.operation === operation, `operation is ${String(operation)}`);
  expect(record, value.ok === false, "ok is false");
  expect(record, value.error.code === code, `error code is ${code}`);
  expect(record, Object.hasOwn(value.error, "details"), "protocol failure has error.details");
  expectNoWarningsField(record, value, "protocol-json stdout");
}

export function expectNoWarningsField(record, value, label) {
  expect(record, !Object.hasOwn(value, "warnings"), `${label} omits warnings`);
}

export function expectStructuredWarning(record, warning, ignoredTokens, kind, reason) {
  expect(record, Boolean(warning), `structured warning exists for ${kind}`);
  expect(
    record,
    JSON.stringify(warning.ignored_tokens) === JSON.stringify(ignoredTokens),
    `${kind} ignored_tokens match`
  );
  expect(record, warning.kind === kind, `${kind} warning kind matches`);
  expect(record, warning.reason === reason, `${kind} warning reason matches`);
}

export function expectStderrWarning(record, ignoredTokens, kind, reason) {
  expectStderrIncludes(record, `ignored_tokens=${JSON.stringify(ignoredTokens)}`);
  expectStderrIncludes(record, `kind=${kind}`);
  expectStderrIncludes(record, `reason=${reason}`);
}

export function expectOutlineResultsEquivalent(record, actual, expected, summary) {
  expect(record, actual?.page === expected?.page, `${summary}: page matches`);
  expect(
    record,
    Array.isArray(actual?.entries) && Array.isArray(expected?.entries),
    `${summary}: entries are arrays`
  );
  expect(record, actual.entries.length === expected.entries.length, `${summary}: entry count matches`);
  for (const index of actual.entries.keys()) {
    expect(record, actual.entries[index].ref === expected.entries[index].ref, `${summary}: entry ${index + 1} ref matches`);
    expect(
      record,
      actual.entries[index].display === expected.entries[index].display,
      `${summary}: entry ${index + 1} display matches`
    );
  }
}

export function expectReadResultsEquivalent(record, actual, expected, summary) {
  for (const field of ["ref", "content", "content_type", "cost", "page"]) {
    expect(record, actual?.[field] === expected?.[field], `${summary}: ${field} matches`);
  }
}

export function expectFindResultsEquivalent(record, actual, expected, summary) {
  expect(record, actual?.page === expected?.page, `${summary}: page matches`);
  expect(
    record,
    Array.isArray(actual?.matches) && Array.isArray(expected?.matches),
    `${summary}: matches are arrays`
  );
  expect(record, actual.matches.length === expected.matches.length, `${summary}: match count matches`);
  for (const index of actual.matches.keys()) {
    expect(record, actual.matches[index].ref === expected.matches[index].ref, `${summary}: match ${index + 1} ref matches`);
    expect(
      record,
      actual.matches[index].display === expected.matches[index].display,
      `${summary}: match ${index + 1} display matches`
    );
  }
}

export function expectInfoResultsEquivalent(record, actual, expected, summary) {
  expect(record, actual?.display === expected?.display, `${summary}: display matches`);
  expect(
    record,
    JSON.stringify(actual?.capabilities) === JSON.stringify(expected?.capabilities),
    `${summary}: capabilities match`
  );
}

export function expectCandidateEvidence(record, candidate, expected) {
  expect(record, Boolean(candidate), `candidate evidence exists for ${expected.adapter_id}`);
  for (const key of ["adapter_id", "stage", "code", "reason", "details"]) {
    expect(record, Object.hasOwn(candidate, key), `candidate evidence has ${key}`);
  }
  for (const [key, value] of Object.entries(expected)) {
    expect(record, candidate[key] === value, `candidate ${key} is ${value}`);
  }
}

export function expectNoJsonPayloadInStderr(record) {
  const jsonLine = record.stderr
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => looksLikeJson(line));
  expect(record, !jsonLine, "stderr does not contain standalone JSON payload");
  expect(
    record,
    !record.stderr.includes("\"protocol_version\"") || !record.stderr.includes("\"ok\""),
    "stderr does not contain protocol response envelope payload"
  );
}

export function expect(record, condition, summary) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

export function assertSetup(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

export function looksLikeJson(value) {
  const trimmed = value.trim();
  return trimmed.startsWith("{") || trimmed.startsWith("[");
}

function findProtocolEnvelopeKeys(value, path = "$") {
  if (Array.isArray(value)) {
    return value.flatMap((item, index) => findProtocolEnvelopeKeys(item, `${path}[${index}]`));
  }
  if (!value || typeof value !== "object") {
    return [];
  }
  const found = [];
  for (const [key, child] of Object.entries(value)) {
    const childPath = `${path}.${key}`;
    if (envelopeKeys.includes(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}

