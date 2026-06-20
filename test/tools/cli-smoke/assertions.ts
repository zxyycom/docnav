import { errorMessage } from "../../../scripts/tools/errors.ts";
import { parseJsonValue } from "../../../scripts/tools/json/value.ts";
import {
  isRecord,
  isStringArray,
  isUnknownArray
} from "../../../scripts/tools/type-guards.ts";
import type { CommandRecord } from "../smoke-harness.ts";

export type JsonRecord = Record<string, unknown>;

const protocolEnvelopeKeys = ["protocol_version", "request_id", "operation", "ok"];

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
  expect(record, record.stdout.includes(value), `stdout includes ${JSON.stringify(value)}`);
}

export function expectStderrIncludes(record: CommandRecord, value: string) {
  expect(record, record.stderr.includes(value), `stderr includes ${JSON.stringify(value)}`);
}

export function expectStdoutWarning(record: CommandRecord, expectedTokens: readonly string[]) {
  expectStdoutIncludes(record, "id=cli_argv_ignored");
  expectStdoutIncludes(record, "effect=operation_continued");
  expectStdoutIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStdoutIncludes(record, JSON.stringify(token));
  }
}

export function expectStderrWarning(record: CommandRecord, expectedTokens: readonly string[]) {
  expectStderrIncludes(record, "id=cli_argv_ignored");
  expectStderrIncludes(record, "effect=operation_continued");
  expectStderrIncludes(record, "details=");
  for (const token of expectedTokens) {
    expectStderrIncludes(record, JSON.stringify(token));
  }
}

export function parseJson(record: CommandRecord): JsonRecord {
  let value: unknown;
  try {
    value = parseJsonValue(record.stdout, "stdout JSON");
    record.assertions.push({ ok: true, summary: "stdout parses as JSON" });
  } catch (error: unknown) {
    record.assertions.push({
      ok: false,
      summary: `stdout parses as JSON: ${errorMessage(error)}`
    });
    throw error;
  }
  return expectJsonObject(record, value, "stdout JSON is an object");
}

export function parseReadableViewHeader(record: CommandRecord, output = record.stdout): JsonRecord {
  const headerBoundary = output.indexOf("\n\n[block ");
  const headerText = output.slice(0, headerBoundary >= 0 ? headerBoundary : output.length);
  let value: unknown;
  try {
    value = parseJsonValue(headerText, "readable-view header JSON");
    record.assertions.push({ ok: true, summary: "readable-view header parses as JSON" });
  } catch (error: unknown) {
    record.assertions.push({
      ok: false,
      summary: `readable-view header parses as JSON: ${errorMessage(error)}`
    });
    throw error;
  }
  return expectJsonObject(record, value, "readable-view header is an object");
}

export function expectNoProtocolEnvelope(record: CommandRecord, value: unknown) {
  const found = findProtocolEnvelopeKeys(value);
  for (const key of protocolEnvelopeKeys) {
    const leakedPaths = found.filter((item) => item.key === key).map((item) => item.path);
    const location = leakedPaths.length > 0 ? ` at ${leakedPaths.join(", ")}` : "";
    expect(record, leakedPaths.length === 0, `readable JSON omits ${key}${location}`);
  }
}

export function expectProtocolSuccess(record: CommandRecord, value: unknown, operation: string) {
  const envelope = expectJsonObject(record, value, "protocol response is an object");
  expect(record, envelope.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof envelope.request_id === "string" && envelope.request_id.length > 0, "request_id is nonempty");
  expect(record, envelope.operation === operation, `operation is ${operation}`);
  expect(record, envelope.ok === true, "ok is true");
  expect(record, Object.hasOwn(envelope, "result"), "protocol success has result");
  expectNoWarningsField(record, envelope, "protocol-json stdout");
}

export function expectProtocolFailure(record: CommandRecord, value: unknown, operation: string | null, code: string) {
  const envelope = expectJsonObject(record, value, "protocol response is an object");
  const error = expectJsonObject(record, envelope.error, "protocol failure has error object");
  expect(record, envelope.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof envelope.request_id === "string" && envelope.request_id.length > 0, "request_id is nonempty");
  expect(record, envelope.operation === operation, `operation is ${operation}`);
  expect(record, envelope.ok === false, "ok is false");
  expect(record, error.code === code, `error code is ${code}`);
  expect(record, Object.hasOwn(error, "details"), "protocol failure has error.details");
  expectNoWarningsField(record, envelope, "protocol-json stdout");
}

export function expectNoWarningsField(record: CommandRecord, value: unknown, label: string) {
  const object = expectJsonObject(record, value, `${label} is an object`);
  expect(record, !Object.hasOwn(object, "warnings"), `${label} omits warnings`);
}

export function expectStructuredWarning(
  record: CommandRecord,
  warning: unknown,
  expectedTokens: readonly string[],
  label = "CLI argv"
) {
  const warningRecord = expectJsonObject(record, warning, `structured warning exists for ${label}`);
  const details = expectJsonObject(record, warningRecord.details, "CLI argv warning details is an object");
  const tokens = expectStringArray(record, details.tokens, "CLI argv warning details.tokens is an array");
  expect(record, warningRecord.id === "cli_argv_ignored", "CLI argv warning id matches");
  expect(record, warningRecord.effect === "operation_continued", "CLI argv warning effect matches");
  expect(
    record,
    typeof warningRecord.reason === "string" && warningRecord.reason.length > 0,
    "CLI argv warning reason is nonempty"
  );
  for (const token of expectedTokens) {
    expect(record, tokens.includes(token), `CLI argv warning mentions ${token}`);
  }
}

export function expectOutlineResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  const actualEntries = expectObjectArray(record, actualObject.entries, `${summary}: actual entries are objects`);
  const expectedEntries = expectObjectArray(record, expectedObject.entries, `${summary}: expected entries are objects`);
  expect(record, actualObject.page === expectedObject.page, `${summary}: page matches`);
  expect(record, actualEntries.length === expectedEntries.length, `${summary}: entry count matches`);
  for (const index of actualEntries.keys()) {
    const actualEntry = actualEntries[index];
    const expectedEntry = expectedEntries[index];
    expect(record, actualEntry.ref === expectedEntry.ref, `${summary}: entry ${index + 1} ref matches`);
    expect(record, actualEntry.display === expectedEntry.display, `${summary}: entry ${index + 1} display matches`);
  }
}

export function expectReadResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  for (const field of ["ref", "content", "content_type", "cost", "page"]) {
    expect(record, actualObject[field] === expectedObject[field], `${summary}: ${field} matches`);
  }
}

export function expectReadableViewBlockRestoresField(
  record: CommandRecord,
  output: string,
  pointer: string,
  expectedPayload: string
) {
  const headerBoundary = output.indexOf("\n\n[block ");
  expect(record, headerBoundary >= 0, `readable-view has block section for ${pointer}`);
  const header = parseReadableViewHeader(record, output);
  const blockRef = expectJsonObject(record, jsonPointer(header, pointer), `readable-view header has ${pointer} block reference`);
  expect(record, blockRef.$block === pointer, `readable-view header has ${pointer} block reference`);
  const expectedBytes = Buffer.byteLength(expectedPayload, "utf8");
  expect(record, blockRef.bytes === expectedBytes, `readable-view header ${pointer} bytes matches payload`);

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

export function expectReadableViewFieldValue(
  record: CommandRecord,
  output: string,
  pointer: string,
  expectedValue: unknown
) {
  const header = parseReadableViewHeader(record, output);
  const actual = jsonPointer(header, pointer);
  expect(record, deepEqual(actual, expectedValue), `readable-view header field ${pointer} matches expected value`);
}

export function expectNoReadableViewBlocks(record: CommandRecord, output = record.stdout, label = "readable-view") {
  expect(record, !output.includes("\n\n[block "), `${label} has no block sections`);
  expect(record, !output.includes("\n[endblock "), `${label} has no end markers`);
}

export function expectFindResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  const actualMatches = expectObjectArray(record, actualObject.matches, `${summary}: actual matches are objects`);
  const expectedMatches = expectObjectArray(record, expectedObject.matches, `${summary}: expected matches are objects`);
  expect(record, actualObject.page === expectedObject.page, `${summary}: page matches`);
  expect(record, actualMatches.length === expectedMatches.length, `${summary}: match count matches`);
  for (const index of actualMatches.keys()) {
    const actualMatch = actualMatches[index];
    const expectedMatch = expectedMatches[index];
    expect(record, actualMatch.ref === expectedMatch.ref, `${summary}: match ${index + 1} ref matches`);
    expect(record, actualMatch.display === expectedMatch.display, `${summary}: match ${index + 1} display matches`);
  }
}

export function expectInfoResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  expect(record, actualObject.display === expectedObject.display, `${summary}: display matches`);
  expect(
    record,
    JSON.stringify(actualObject.capabilities) === JSON.stringify(expectedObject.capabilities),
    `${summary}: capabilities match`
  );
}

export function expectNoJsonPayloadInStderr(record: CommandRecord) {
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
  if (isJsonRecord(value)) {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return {};
}

export function expectObjectArray(record: CommandRecord, value: unknown, summary: string): JsonRecord[] {
  if (isUnknownArray(value) && value.every(isJsonRecord)) {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return [];
}

export function expectString(record: CommandRecord, value: unknown, summary: string): string {
  if (typeof value === "string") {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return "";
}

export function expectNumber(record: CommandRecord, value: unknown, summary: string): number {
  if (typeof value === "number") {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return 0;
}

export function expectStringArray(record: CommandRecord, value: unknown, summary: string): string[] {
  if (isStringArray(value)) {
    expect(record, true, summary);
    return value;
  }
  expect(record, false, summary);
  return [];
}

export function looksLikeJson(value: string) {
  const trimmed = value.trim();
  return trimmed.startsWith("{") || trimmed.startsWith("[");
}

export function containsProtocolResponseEnvelope(value: string) {
  return value.includes("\"protocol_version\"") && value.includes("\"ok\"");
}

function findProtocolEnvelopeKeys(value: unknown, path = "$"): { key: string; path: string }[] {
  if (isUnknownArray(value)) {
    return value.flatMap((item, index) => findProtocolEnvelopeKeys(item, `${path}[${index}]`));
  }
  if (!isJsonRecord(value)) {
    return [];
  }
  const found: { key: string; path: string }[] = [];
  for (const [key, child] of Object.entries(value)) {
    const childPath = `${path}.${key}`;
    if (protocolEnvelopeKeys.includes(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}

function jsonPointer(value: unknown, pointer: string): unknown {
  if (pointer === "") {
    return value;
  }
  let current = value;
  for (const segment of pointer.split("/").slice(1)) {
    if (!isRecord(current)) {
      return undefined;
    }
    const key = segment.replaceAll("~1", "/").replaceAll("~0", "~");
    current = current[key];
  }
  return current;
}

function deepEqual(actual: unknown, expected: unknown): boolean {
  if (Object.is(actual, expected)) {
    return true;
  }
  if (isUnknownArray(actual) && isUnknownArray(expected)) {
    return actual.length === expected.length && actual.every((item, index) => deepEqual(item, expected[index]));
  }
  if (isJsonRecord(actual) && isJsonRecord(expected)) {
    const actualKeys = Object.keys(actual);
    const expectedKeys = Object.keys(expected);
    return (
      actualKeys.length === expectedKeys.length &&
      expectedKeys.every((key) => Object.hasOwn(actual, key) && deepEqual(actual[key], expected[key]))
    );
  }
  return false;
}

function isJsonRecord(value: unknown): value is JsonRecord {
  return isRecord(value) && !isUnknownArray(value);
}
