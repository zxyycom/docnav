import {
  assert,
  assertDeepEqual,
} from "../assertions.ts";
import { listExampleJson, readJson } from "../json/files.ts";
import {
  EXAMPLES,
  FIELDS,
  MARKDOWN_MANIFEST_EXPECTED,
  MCP_EXAMPLE_FILE,
  OPERATION_NAMES,
  OPERATIONS,
  PROTOCOL_EXAMPLE_FILE,
  READABLE_EXAMPLE_FILE,
  REQUIRED_ERROR_DETAILS_BY_CODE,
} from "../config.ts";
import { isRecord } from "../../type-guards.ts";

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
    FIELDS.display in result &&
    Array.isArray(result[FIELDS.capabilities]) &&
    !(FIELDS.page in result),
};

function toReadablePayload(_operation: string, protocolResult: unknown): unknown {
  return protocolResult;
}

function validateProtocolPair(operation: string) {
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

function validateExampleBudget(operation: string, request: Record<string, unknown>, result: Record<string, unknown>): void {
  const requestArgs = jsonObject(request[FIELDS.arguments], `${operation} request arguments`);
  const limit = requestArgs[FIELDS.limitChars];
  if (typeof limit !== "number") {
    return;
  }

  if (operation === OPERATION_NAMES.read) {
    const content = result[FIELDS.content];
    assert(typeof content === "string", `${operation} content must be a string`);
    assert(
      codePointLength(content) <= limit,
      `${operation} content exceeds limit_chars in example`,
    );
    return;
  }

  const records =
    operation === OPERATION_NAMES.outline
      ? jsonArray(result[FIELDS.entries], `${operation} entries`)
      : jsonArray(result[FIELDS.matches], `${operation} matches`);
  const recordSizes = records.map((record, index) => {
    const recordObject = jsonObject(record, `${operation} record ${index}`);
    const ref = recordObject[FIELDS.ref];
    const display = recordObject[FIELDS.display];
    assert(typeof ref === "string", `${operation} record ${index} ref must be a string`);
    assert(typeof display === "string", `${operation} record ${index} display must be a string`);
    return {
      ref: codePointLength(ref),
      display: codePointLength(display),
    };
  });
  const totalChars = recordSizes.reduce(
    (sum, record) => sum + record.ref + record.display,
    0,
  );
  if (totalChars <= limit) {
    return;
  }

  const oversizedRefRecords = recordSizes.filter(
    (record) => record.ref > limit,
  );
  assert(
    records.length === 1 && oversizedRefRecords.length === 1,
    `${operation} ref + display exceeds limit_chars in example without single oversized ref exception`,
  );
  assert(
    recordSizes[0].display > 0 && recordSizes[0].display <= limit,
    `${operation} oversized ref example display must be readable and fit limit_chars`,
  );
}

function validateProtocolReadableMappings() {
  for (const operation of OPERATIONS) {
    const { request, response } = validateProtocolPair(operation);
    const result = jsonObject(response[FIELDS.result], `${operation} response result`);
    validateExampleBudget(operation, request, result);

    const readable = readJson(READABLE_EXAMPLE_FILE.result(operation));
    assertDeepEqual(
      readable,
      toReadablePayload(operation, response[FIELDS.result]),
      `${operation} readable JSON must preserve protocol result semantics`,
    );

    const mcp = jsonObject(readJson(MCP_EXAMPLE_FILE.response(operation)), `${operation} MCP response`);
    const mcpResult = jsonObject(mcp[FIELDS.result], `${operation} MCP result`);
    assertDeepEqual(
      mcpResult[FIELDS.structuredContent],
      readable,
      `${operation} MCP structuredContent must match readable JSON example`,
    );
  }

  console.log(
    `protocol/readable mapping ok: ${OPERATIONS.length} operation(s)`,
  );
}

function validateErrorDetails() {
  const errorFiles = listExampleJson(/^error-.*\.json$/);
  for (const errorRelPath of errorFiles) {
    const response = jsonObject(readJson(errorRelPath), errorRelPath);
    assert(
      response[FIELDS.ok] === false,
      `${errorRelPath} must be an error response`,
    );
    assert(
      !(FIELDS.result in response),
      `${errorRelPath} error response must not include result`,
    );
    const responseOperation = response[FIELDS.operation];
    assert(
      responseOperation === null ||
        (typeof responseOperation === "string" && OPERATIONS.includes(responseOperation)),
      `${errorRelPath} error operation must be known operation or null`,
    );
    const error = jsonObject(response[FIELDS.error], `${errorRelPath} error`);
    const details = jsonObject(error[FIELDS.details], `${errorRelPath} error details`);
    const errorCode = error[FIELDS.code];
    assert(typeof errorCode === "string", `${errorRelPath} error code must be a string`);
    const requiredDetails = (REQUIRED_ERROR_DETAILS_BY_CODE as Record<string, readonly string[]>)[errorCode];
    assert(
      requiredDetails,
      `${errorRelPath} uses unknown error code ${errorCode}`,
    );
    for (const field of requiredDetails) {
      assert(
        field in details,
        `${errorRelPath} missing error.details.${field}`,
      );
    }
  }

  const readableError = jsonObject(readJson(EXAMPLES.readableError), EXAMPLES.readableError);
  const readableErrorCode = readableError[FIELDS.code];
  assert(typeof readableErrorCode === "string", "readable-error.json code must be a string");
  assert(
    readableErrorCode in REQUIRED_ERROR_DETAILS_BY_CODE,
    "readable-error.json uses unknown error code",
  );
  for (const field of (REQUIRED_ERROR_DETAILS_BY_CODE as Record<string, readonly string[]>)[
    readableErrorCode
  ]) {
    assert(
      field in jsonObject(readableError[FIELDS.details] ?? {}, "readable-error.json details"),
      `readable-error.json missing details.${field}`,
    );
  }

  console.log(`error details ok: ${errorFiles.length + 1} file(s)`);
}

function validateManifestSemantics() {
  const manifest = jsonObject(readJson(EXAMPLES.manifest), EXAMPLES.manifest);
  const adapter = jsonObject(manifest[FIELDS.adapter], "manifest adapter");
  assert(
    adapter[FIELDS.id] ===
      MARKDOWN_MANIFEST_EXPECTED.adapterId,
    "manifest example must describe docnav-markdown",
  );

  const capabilities = jsonArray(manifest[FIELDS.capabilities], "manifest capabilities");
  for (const capability of MARKDOWN_MANIFEST_EXPECTED.capabilities) {
    assert(
      capabilities.includes(capability),
      `markdown manifest example missing capability ${capability}`,
    );
  }

  const formats = jsonArray(manifest[FIELDS.formats], "manifest formats");
  const markdownFormat = formats.find(
    (format): format is Record<string, unknown> =>
      isRecord(format) && format[FIELDS.id] === MARKDOWN_MANIFEST_EXPECTED.formatId,
  );
  assert(markdownFormat, "manifest example missing markdown format");
  const extensions = jsonArray(markdownFormat[FIELDS.extensions], "markdown format extensions");
  assert(
    extensions.includes(
      MARKDOWN_MANIFEST_EXPECTED.extension,
    ),
    "markdown manifest example missing .md extension",
  );
  const contentTypes = jsonArray(markdownFormat[FIELDS.contentTypes], "markdown format content_types");
  assert(
    contentTypes.includes(
      MARKDOWN_MANIFEST_EXPECTED.contentType,
    ),
    "markdown manifest example missing text/markdown content type",
  );

  console.log("manifest example consistency ok: markdown capabilities and format");
}

export function validateProtocolExampleSemantics() {
  validateProtocolReadableMappings();
  validateErrorDetails();
  validateManifestSemantics();
}

function jsonObject(value: unknown, label: string): Record<string, unknown> {
  assert(isRecord(value), `${label} must be an object`);
  return value;
}

function jsonArray(value: unknown, label: string): unknown[] {
  assert(Array.isArray(value), `${label} must be an array`);
  return value;
}

function codePointLength(value: string): number {
  return [...value].length;
}
