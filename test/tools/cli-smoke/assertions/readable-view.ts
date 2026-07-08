import { isRecord, isUnknownArray } from "../../../../scripts/tools/foundation/src/type-guards.ts";
import type { CommandRecord } from "../../smoke-harness.ts";
import {
  expect,
  expectJsonObject,
  isJsonRecord,
  parseJsonObjectFromText
} from "./base.ts";

interface ReadableViewBlockExpectation {
  expectedBytes: number;
  expectedPayload: string;
  headerBoundary: number;
  pointer: string;
}

export function parseReadableViewHeader(record: CommandRecord, output = record.stdout) {
  const headerText = readableViewHeaderText(output);
  return parseJsonObjectFromText(
    record,
    headerText,
    "readable-view header JSON",
    "readable-view header parses as JSON",
    "readable-view header is an object"
  );
}

export function expectReadableViewBlockRestoresField(
  record: CommandRecord,
  output: string,
  pointer: string,
  expectedPayload: string
) {
  const headerBoundary = output.indexOf("\n\n[block ");
  expect(record, headerBoundary >= 0, `readable-view has block section for ${pointer}`);

  const expectedBytes = Buffer.byteLength(expectedPayload, "utf8");
  expectReadableViewBlockReference(record, output, pointer, expectedBytes);
  expectReadableViewBlockPayload(record, output, {
    expectedBytes,
    expectedPayload,
    headerBoundary,
    pointer
  });
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

function expectReadableViewBlockReference(
  record: CommandRecord,
  output: string,
  pointer: string,
  expectedBytes: number
) {
  const header = parseReadableViewHeader(record, output);
  const blockRef = expectJsonObject(record, jsonPointer(header, pointer), `readable-view header has ${pointer} block reference`);
  expect(record, blockRef.$block === pointer, `readable-view header has ${pointer} block reference`);
  expect(record, blockRef.bytes === expectedBytes, `readable-view header ${pointer} bytes matches payload`);
}

function expectReadableViewBlockPayload(
  record: CommandRecord,
  output: string,
  expectation: ReadableViewBlockExpectation
) {
  const { expectedBytes, expectedPayload, headerBoundary, pointer } = expectation;
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

  const markerStart = expectReadableViewBlockFraming(record, outputBytes, payloadEnd, pointer, expectedPayload);
  const endMarkerBytes = Buffer.from(`[endblock ${pointer}]\n`, "utf8");
  expect(
    record,
    outputBytes.subarray(markerStart, markerStart + endMarkerBytes.length).equals(endMarkerBytes),
    `readable-view block ${pointer} end marker follows declared payload bytes`
  );
}

function expectReadableViewBlockFraming(
  record: CommandRecord,
  outputBytes: Buffer,
  payloadEnd: number,
  pointer: string,
  expectedPayload: string
) {
  if (expectedPayload.endsWith("\n")) {
    return payloadEnd;
  }
  expect(record, outputBytes[payloadEnd] === 0x0a, `readable-view block ${pointer} has framing LF before end marker`);
  return payloadEnd + 1;
}

function readableViewHeaderText(output: string) {
  const headerBoundary = output.indexOf("\n\n[block ");
  return output.slice(0, headerBoundary >= 0 ? headerBoundary : output.length);
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
