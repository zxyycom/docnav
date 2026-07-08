import { assert } from "../../assertions.ts";
import { FIELDS, PROTOCOL_EXAMPLE_FILE } from "../../config.ts";
import { readJson } from "../../json/files.ts";
import { isRecord } from "../../../foundation/src/type-guards.ts";
import { jsonObject } from "./json.ts";

type ProtocolResultCheck = (result: Record<string, unknown>) => boolean;

const PROTOCOL_RESULT_CHECKS: Partial<Record<string, ProtocolResultCheck>> = {
  outline: (result) =>
    Array.isArray(result[FIELDS.entries]) &&
    FIELDS.page in result &&
    !(FIELDS.matches in result),
  read: (result) =>
    FIELDS.ref in result &&
    FIELDS.content in result &&
    FIELDS.contentType in result &&
    FIELDS.cost in result,
  find: (result) =>
    Array.isArray(result[FIELDS.matches]) &&
    FIELDS.page in result &&
    !(FIELDS.entries in result),
  info: (result) =>
    (FIELDS.document in result || FIELDS.adapter in result || FIELDS.metadata in result) &&
    !(FIELDS.display in result) &&
    !(FIELDS.page in result),
};

export interface ProtocolExamplePair {
  request: Record<string, unknown>;
  response: Record<string, unknown>;
}

export function validateProtocolPair(operation: string): ProtocolExamplePair {
  const request = jsonObject(readJson(PROTOCOL_EXAMPLE_FILE.request(operation)), `${operation} request`);
  const response = jsonObject(readJson(PROTOCOL_EXAMPLE_FILE.response(operation)), `${operation} response`);

  assert(
    response[FIELDS.protocolVersion] === request[FIELDS.protocolVersion],
    `${operation} response protocol_version must match request`,
  );
  assert(
    response[FIELDS.requestId] === request[FIELDS.requestId],
    `${operation} response request_id must match request`,
  );
  assert(
    response[FIELDS.operation] === request[FIELDS.operation],
    `${operation} response operation must match request`,
  );
  assert(
    response[FIELDS.ok] === true,
    `${operation} protocol response must be successful`,
  );
  validateProtocolResultBinding(
    operation,
    response,
    PROTOCOL_EXAMPLE_FILE.responseName(operation),
  );

  const requestArgs = jsonObject(request[FIELDS.arguments], `${operation} request arguments`);
  const result = jsonObject(response[FIELDS.result], `${operation} response result`);
  const requestedPage = requestArgs[FIELDS.page];
  const returnedPage = result[FIELDS.page];
  if (typeof requestedPage === "number" && returnedPage !== null) {
    assert(
      returnedPage === requestedPage + 1,
      `${operation} response page must be request page + 1`,
    );
  }

  return { request, response };
}

function validateProtocolResultBinding(operation: string, response: Record<string, unknown>, label: string): void {
  const result = response[FIELDS.result];
  assert(
    isRecord(result),
    `${label} missing result object`,
  );

  assert(
    PROTOCOL_RESULT_CHECKS[operation]?.(result),
    `${label} result does not match ${operation}`,
  );
}
