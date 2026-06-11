import {
  envelopeKeys,
  expectedNormalFindDisplayKeywords,
  expectedNormalFindMatchCount
} from "./config.mjs";

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

export function expectNoWarningsField(record, value, label) {
  expect(record, !Object.hasOwn(value, "warnings"), `${label} omits warnings`);
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
  expect(record, value.operation === operation, `operation is ${operation}`);
  expect(record, value.ok === true, "ok is true");
  expect(record, Object.hasOwn(value, "result"), "protocol success has result");
}

export function expectProtocolFailure(record, value, operation, code) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, value.operation === operation, `operation is ${operation}`);
  expect(record, value.ok === false, "ok is false");
  expect(record, value.error.code === code, `error code is ${code}`);
  expect(record, Object.hasOwn(value.error, "details"), "protocol failure has error.details");
}

export function expectNormalFindResult(record, result, label) {
  expect(record, Array.isArray(result.matches), `${label} has matches array`);
  expect(
    record,
    result.matches.length === expectedNormalFindMatchCount,
    `${label} returns exactly ${expectedNormalFindMatchCount} matches`
  );
  const refs = result.matches.map((match) => match?.ref);
  expect(
    record,
    refs.every((ref) => typeof ref === "string" && ref.length > 0),
    `${label} match refs are nonempty`
  );
  expect(record, new Set(refs).size === refs.length, `${label} match refs are unique`);
  for (const [index, actual] of result.matches.entries()) {
    expect(
      record,
      typeof actual.display === "string" && actual.display.length > 0,
      `${label} match ${index + 1} display is nonempty`
    );
    for (const keyword of expectedNormalFindDisplayKeywords) {
      expect(
        record,
        actual.display.includes(keyword),
        `${label} match ${index + 1} display includes ${keyword}`
      );
    }
  }
  expect(record, result.page === null, `${label} page is null`);
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
    expect(
      record,
      actual.matches[index].ref === expected.matches[index].ref,
      `${summary}: match ${index + 1} ref matches`
    );
    expect(
      record,
      actual.matches[index].display === expected.matches[index].display,
      `${summary}: match ${index + 1} display matches`
    );
  }
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
    if (envelopeKeys.includes(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}
