import { isUnknownArray } from "../../../../scripts/tools/foundation/src/type-guards.ts";
import type { CommandRecord } from "../../smoke-harness.ts";
import {
  expect,
  expectJsonObject,
  expectStringArray,
  isJsonRecord
} from "./base.ts";
import type { JsonRecord } from "./base.ts";

type ProtocolEnvelopeKey = "protocol_version" | "request_id" | "operation" | "ok";
type ProtocolEnvelopeKeyLocation = { key: ProtocolEnvelopeKey; path: string };

const protocolEnvelopeKeys: readonly ProtocolEnvelopeKey[] = ["protocol_version", "request_id", "operation", "ok"];

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
  expectProtocolEnvelopeFields(record, envelope, operation, true);
  expect(record, Object.hasOwn(envelope, "result"), "protocol success has result");
}

export function expectProtocolFailure(record: CommandRecord, value: unknown, operation: string | null, code: string) {
  const envelope = expectJsonObject(record, value, "protocol response is an object");
  const error = expectJsonObject(record, envelope.error, "protocol failure has error object");
  expectProtocolEnvelopeFields(record, envelope, operation, false);
  expect(record, error.code === code, `error code is ${code}`);
  expect(record, Object.hasOwn(error, "details"), "protocol failure has error.details");
  expectPrimaryDiagnosticProjection(record, error, "protocol failure");
  return error;
}

export function expectReadableFailure(record: CommandRecord, value: unknown, code: string) {
  const error = expectJsonObject(record, value, "readable failure is an object");
  expectNoProtocolEnvelope(record, error);
  expect(record, error.code === code, `readable failure code is ${code}`);
  expect(record, Object.hasOwn(error, "details"), "readable failure has details");
  expectPrimaryDiagnosticProjection(record, error, "readable failure");
  return error;
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

export function looksLikeJson(value: string) {
  const trimmed = value.trim();
  return trimmed.startsWith("{") || trimmed.startsWith("[");
}

export function containsProtocolResponseEnvelope(value: string) {
  return value.includes("\"protocol_version\"") && value.includes("\"ok\"");
}

function expectProtocolEnvelopeFields(record: CommandRecord, envelope: JsonRecord, operation: string | null, ok: boolean) {
  expect(record, envelope.protocol_version === "0.1", "protocol_version is 0.1");
  expect(record, typeof envelope.request_id === "string" && envelope.request_id.length > 0, "request_id is nonempty");
  expect(record, envelope.operation === operation, `operation is ${operation}`);
  expect(record, envelope.ok === ok, `ok is ${ok}`);
}

function expectPrimaryDiagnosticProjection(record: CommandRecord, error: JsonRecord, label: string) {
  expect(record, typeof error.owner === "string" && error.owner.length > 0, `${label} has nonempty owner`);
  const guidance = expectStringArray(record, error.guidance, `${label} guidance is an array`);
  expect(record, guidance.length > 0, `${label} guidance is nonempty`);
  for (const [index, item] of guidance.entries()) {
    expect(record, item.length > 0, `${label} guidance ${index + 1} is nonempty`);
  }
}

function findProtocolEnvelopeKeys(value: unknown, path = "$"): ProtocolEnvelopeKeyLocation[] {
  if (isUnknownArray(value)) {
    return value.flatMap((item, index) => findProtocolEnvelopeKeys(item, `${path}[${index}]`));
  }
  if (!isJsonRecord(value)) {
    return [];
  }

  const found: ProtocolEnvelopeKeyLocation[] = [];
  for (const [key, child] of Object.entries(value)) {
    const childPath = `${path}.${key}`;
    if (isProtocolEnvelopeKey(key)) {
      found.push({ key, path: childPath });
    }
    found.push(...findProtocolEnvelopeKeys(child, childPath));
  }
  return found;
}

function isProtocolEnvelopeKey(key: string): key is ProtocolEnvelopeKey {
  return protocolEnvelopeKeys.includes(key as ProtocolEnvelopeKey);
}
