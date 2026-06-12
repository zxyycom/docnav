const protocolEnvelopeKeys = ["protocol_version", "request_id", "operation", "ok"];

export function expectExit(record, expected) {
  expect(record, record.exitCode === expected, `exit code is ${expected}`);
}

export function expectStdoutEmpty(record) {
  expect(record, record.stdout.length === 0, "stdout is empty");
}

export function expectStderrEmpty(record) {
  expect(record, record.stderr.length === 0, "stderr is empty");
}

export function expectStdoutIncludes(record, value) {
  expect(record, record.stdout.includes(value), `stdout includes ${JSON.stringify(value)}`);
}

export function expectStderrIncludes(record, value) {
  expect(record, record.stderr.includes(value), `stderr includes ${JSON.stringify(value)}`);
}

export function expectStdoutWarning(record, expectedTokens) {
  expectStdoutIncludes(record, "id=cli_argv_ignored");
  expectStdoutIncludes(record, "effect=operation_continued");
  expectStdoutIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStdoutIncludes(record, JSON.stringify(token));
  }
}

export function expectStderrWarning(record, expectedTokens) {
  expectStderrIncludes(record, "id=cli_argv_ignored");
  expectStderrIncludes(record, "effect=operation_continued");
  expectStderrIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStderrIncludes(record, JSON.stringify(token));
  }
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
  for (const key of protocolEnvelopeKeys) {
    const leakedPaths = found.filter((item) => item.key === key).map((item) => item.path);
    const location = leakedPaths.length > 0 ? ` at ${leakedPaths.join(", ")}` : "";
    expect(record, leakedPaths.length === 0, `readable JSON omits ${key}${location}`);
  }
}

export function expectProtocolSuccess(record, value, operation) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof value.request_id === "string" && value.request_id.length > 0, "request_id is nonempty");
  expect(record, value.operation === operation, `operation is ${String(operation)}`);
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

export function expectStructuredWarning(record, warning, expectedTokens, label = "CLI argv") {
  expect(record, Boolean(warning), `structured warning exists for ${label}`);
  expect(record, warning.id === "cli_argv_ignored", "CLI argv warning id matches");
  expect(record, warning.effect === "operation_continued", "CLI argv warning effect matches");
  expect(record, typeof warning.reason === "string" && warning.reason.length > 0, "CLI argv warning reason is nonempty");
  expect(record, Array.isArray(warning.details?.tokens), "CLI argv warning details.tokens is an array");
  for (const token of expectedTokens) {
    expect(record, warning.details.tokens.includes(token), `CLI argv warning mentions ${token}`);
  }
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

export function expectNoJsonPayloadInStderr(record) {
  const jsonLine = record.stderr
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => looksLikeJson(line));
  expect(record, !jsonLine, "stderr does not contain standalone JSON payload");
  expect(
    record,
    !containsProtocolResponseEnvelope(record.stderr),
    "stderr does not contain protocol response envelope payload"
  );
}

export function expectIncludes(record, values, value, summary) {
  expect(record, Array.isArray(values) && values.includes(value), summary);
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

export function containsProtocolResponseEnvelope(value) {
  return value.includes("\"protocol_version\"") && value.includes("\"ok\"");
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
    if (protocolEnvelopeKeys.includes(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}
