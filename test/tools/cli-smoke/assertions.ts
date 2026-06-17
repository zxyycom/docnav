const protocolEnvelopeKeys = ["protocol_version", "request_id", "operation", "ok"];

export function expectExit(record: any, expected: any) {
  expect(record, record.exitCode === expected, `exit code is ${expected}`);
}

export function expectStdoutEmpty(record: any) {
  expect(record, record.stdout.length === 0, "stdout is empty");
}

export function expectStderrEmpty(record: any) {
  expect(record, record.stderr.length === 0, "stderr is empty");
}

export function expectStdoutIncludes(record: any, value: any) {
  expect(record, record.stdout.includes(value), `stdout includes ${JSON.stringify(value)}`);
}

export function expectStderrIncludes(record: any, value: any) {
  expect(record, record.stderr.includes(value), `stderr includes ${JSON.stringify(value)}`);
}

export function expectStdoutWarning(record: any, expectedTokens: any) {
  expectStdoutIncludes(record, "id=cli_argv_ignored");
  expectStdoutIncludes(record, "effect=operation_continued");
  expectStdoutIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStdoutIncludes(record, JSON.stringify(token));
  }
}

export function expectStderrWarning(record: any, expectedTokens: any) {
  expectStderrIncludes(record, "id=cli_argv_ignored");
  expectStderrIncludes(record, "effect=operation_continued");
  expectStderrIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStderrIncludes(record, JSON.stringify(token));
  }
}

export function parseJson(record: any) {
  try {
    const value = JSON.parse(record.stdout);
    record.assertions.push({ ok: true, summary: "stdout parses as JSON" });
    return value;
  } catch (error: any) {
    record.assertions.push({
      ok: false,
      summary: `stdout parses as JSON: ${error.message}`
    });
    throw error;
  }
}

export function parseReadableViewHeader(record: any, output = record.stdout) {
  const headerBoundary = output.indexOf("\n\n[block ");
  const headerText = output.slice(0, headerBoundary >= 0 ? headerBoundary : output.length);
  try {
    const value = JSON.parse(headerText);
    record.assertions.push({ ok: true, summary: "readable-view header parses as JSON" });
    return value;
  } catch (error: any) {
    record.assertions.push({
      ok: false,
      summary: `readable-view header parses as JSON: ${error.message}`
    });
    throw error;
  }
}

export function expectNoProtocolEnvelope(record: any, value: any) {
  const found = findProtocolEnvelopeKeys(value);
  for (const key of protocolEnvelopeKeys) {
    const leakedPaths = found.filter((item: any) => item.key === key).map((item: any) => item.path);
    const location = leakedPaths.length > 0 ? ` at ${leakedPaths.join(", ")}` : "";
    expect(record, leakedPaths.length === 0, `readable JSON omits ${key}${location}`);
  }
}

export function expectProtocolSuccess(record: any, value: any, operation: any) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof value.request_id === "string" && value.request_id.length > 0, "request_id is nonempty");
  expect(record, value.operation === operation, `operation is ${String(operation)}`);
  expect(record, value.ok === true, "ok is true");
  expect(record, Object.hasOwn(value, "result"), "protocol success has result");
  expectNoWarningsField(record, value, "protocol-json stdout");
}

export function expectProtocolFailure(record: any, value: any, operation: any, code: any) {
  expect(record, value.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof value.request_id === "string" && value.request_id.length > 0, "request_id is nonempty");
  expect(record, value.operation === operation, `operation is ${String(operation)}`);
  expect(record, value.ok === false, "ok is false");
  expect(record, value.error.code === code, `error code is ${code}`);
  expect(record, Object.hasOwn(value.error, "details"), "protocol failure has error.details");
  expectNoWarningsField(record, value, "protocol-json stdout");
}

export function expectNoWarningsField(record: any, value: any, label: any) {
  expect(record, !Object.hasOwn(value, "warnings"), `${label} omits warnings`);
}

export function expectStructuredWarning(record: any, warning: any, expectedTokens: any, label = "CLI argv") {
  expect(record, Boolean(warning), `structured warning exists for ${label}`);
  expect(record, warning.id === "cli_argv_ignored", "CLI argv warning id matches");
  expect(record, warning.effect === "operation_continued", "CLI argv warning effect matches");
  expect(record, typeof warning.reason === "string" && warning.reason.length > 0, "CLI argv warning reason is nonempty");
  expect(record, Array.isArray(warning.details?.tokens), "CLI argv warning details.tokens is an array");
  for (const token of expectedTokens) {
    expect(record, warning.details.tokens.includes(token), `CLI argv warning mentions ${token}`);
  }
}

export function expectOutlineResultsEquivalent(record: any, actual: any, expected: any, summary: any) {
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

export function expectReadResultsEquivalent(record: any, actual: any, expected: any, summary: any) {
  for (const field of ["ref", "content", "content_type", "cost", "page"]) {
    expect(record, actual?.[field] === expected?.[field], `${summary}: ${field} matches`);
  }
}

export function expectReadableViewBlockRestoresField(record: any, output: any, pointer: any, expectedPayload: any) {
  const headerBoundary = output.indexOf("\n\n[block ");
  expect(record, headerBoundary >= 0, `readable-view has block section for ${pointer}`);
  const header = parseReadableViewHeader(record, output);
  const blockRef = jsonPointer(header, pointer);
  expect(record, blockRef?.$block === pointer, `readable-view header has ${pointer} block reference`);
  const expectedBytes = Buffer.byteLength(expectedPayload, "utf8");
  expect(record, blockRef?.bytes === expectedBytes, `readable-view header ${pointer} bytes matches payload`);

  const startMarker = `[block ${pointer} bytes=${expectedBytes}]\n`;
  const outputBytes = Buffer.from(output, "utf8");
  const startMarkerBytes = Buffer.from(startMarker, "utf8");
  const headerBoundaryBytes = Buffer.byteLength(output.slice(0, headerBoundary), "utf8");
  const start = outputBytes.indexOf(startMarkerBytes, headerBoundaryBytes);
  expect(record, start >= 0, `readable-view block start marker matches ${pointer} bytes`);
  const payloadStart = start + startMarkerBytes.length;
  const payloadEnd = payloadStart + expectedBytes;
  const payload = outputBytes.subarray(payloadStart, payloadEnd).toString("utf8");
  expect(record, payload === expectedPayload, `readable-view block ${pointer} restores readable-json payload`);

  let markerStart = payloadEnd;
  if (!expectedPayload.endsWith("\n")) {
    expect(record, outputBytes[markerStart] === 0x0a, `readable-view block ${pointer} has framing LF before end marker`);
    markerStart += 1;
  }
  const endMarkerBytes = Buffer.from(`[endblock ${pointer}]\n`, "utf8");
  expect(
    record,
    outputBytes.subarray(markerStart, markerStart + endMarkerBytes.length).equals(endMarkerBytes),
    `readable-view block ${pointer} end marker follows declared payload bytes`
  );
}

export function expectReadableViewFieldValue(record: any, output: any, pointer: any, expectedValue: any) {
  const header = parseReadableViewHeader(record, output);
  const actual = jsonPointer(header, pointer);
  expect(record, deepEqual(actual, expectedValue), `readable-view header field ${pointer} matches expected value`);
}

export function expectNoReadableViewBlocks(record: any, output = record.stdout, label = "readable-view") {
  expect(record, !output.includes("\n\n[block "), `${label} has no block sections`);
  expect(record, !output.includes("\n[endblock "), `${label} has no end markers`);
}

export function expectFindResultsEquivalent(record: any, actual: any, expected: any, summary: any) {
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

export function expectInfoResultsEquivalent(record: any, actual: any, expected: any, summary: any) {
  expect(record, actual?.display === expected?.display, `${summary}: display matches`);
  expect(
    record,
    JSON.stringify(actual?.capabilities) === JSON.stringify(expected?.capabilities),
    `${summary}: capabilities match`
  );
}

export function expectNoJsonPayloadInStderr(record: any) {
  const jsonLine = record.stderr
    .split(/\r?\n/)
    .map((line: any) => line.trim())
    .find((line: any) => looksLikeJson(line));
  expect(record, !jsonLine, "stderr does not contain standalone JSON payload");
  expect(
    record,
    !containsProtocolResponseEnvelope(record.stderr),
    "stderr does not contain protocol response envelope payload"
  );
}

export function expectIncludes(record: any, values: any, value: any, summary: any) {
  expect(record, Array.isArray(values) && values.includes(value), summary);
}

export function expect(record: any, condition: any, summary: any) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

export function assertSetup(condition: any, message: any) {
  if (!condition) {
    throw new Error(message);
  }
}

export function looksLikeJson(value: any) {
  const trimmed = value.trim();
  return trimmed.startsWith("{") || trimmed.startsWith("[");
}

export function containsProtocolResponseEnvelope(value: any) {
  return value.includes("\"protocol_version\"") && value.includes("\"ok\"");
}

function findProtocolEnvelopeKeys(value: any, path = "$"): any[] {
  if (Array.isArray(value)) {
    return value.flatMap((item, index) => findProtocolEnvelopeKeys(item, `${path}[${index}]`));
  }
  if (!value || typeof value !== "object") {
    return [];
  }
  const found: any[] = [];
  for (const [key, child] of Object.entries(value)) {
    const childPath = `${path}.${key}`;
    if (protocolEnvelopeKeys.includes(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}

function jsonPointer(value: any, pointer: any) {
  if (pointer === "") {
    return value;
  }
  return pointer
    .split("/")
    .slice(1)
    .reduce((current: any, segment: any) => {
      if (current === undefined || current === null) {
        return undefined;
      }
      const key = segment.replaceAll("~1", "/").replaceAll("~0", "~");
      return current[key];
    }, value);
}

function deepEqual(actual: any, expected: any): boolean {
  if (Object.is(actual, expected)) {
    return true;
  }
  if (Array.isArray(actual) && Array.isArray(expected)) {
    return actual.length === expected.length && actual.every((item, index) => deepEqual(item, expected[index]));
  }
  if (
    actual &&
    expected &&
    typeof actual === "object" &&
    typeof expected === "object" &&
    !Array.isArray(actual) &&
    !Array.isArray(expected)
  ) {
    const actualKeys = Object.keys(actual);
    const expectedKeys = Object.keys(expected);
    return (
      actualKeys.length === expectedKeys.length &&
      expectedKeys.every((key) => Object.hasOwn(actual, key) && deepEqual(actual[key], expected[key]))
    );
  }
  return false;
}
