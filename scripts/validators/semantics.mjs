import {
  assert,
  assertDeepEqual,
  charLength,
  listExampleJson,
  readJson
} from "./fs-utils.mjs";
import {
  EXAMPLES,
  FIELDS,
  MARKDOWN_MANIFEST_EXPECTED,
  MCP_EXAMPLE_FILE,
  OPERATION_NAMES,
  OPERATIONS,
  PROTOCOL_EXAMPLE_FILE,
  READABLE_EXAMPLE_FILE,
  REQUIRED_ERROR_DETAILS_BY_CODE
} from "./config.mjs";

function toReadablePayload(operation, protocolResult) {
  return protocolResult;
}

function validateProtocolPair(operation) {
  const request = readJson(PROTOCOL_EXAMPLE_FILE.request(operation));
  const response = readJson(PROTOCOL_EXAMPLE_FILE.response(operation));

  assert(
    response[FIELDS.protocolVersion] === request[FIELDS.protocolVersion],
    `${operation} response protocol_version must match request`
  );
  assert(
    response[FIELDS.requestId] === request[FIELDS.requestId],
    `${operation} response request_id must match request`
  );
  assert(
    response[FIELDS.operation] === request[FIELDS.operation],
    `${operation} response operation must match request`
  );
  assert(response[FIELDS.ok] === true, `${operation} protocol response must be successful`);
  validateProtocolResultBinding(operation, response, PROTOCOL_EXAMPLE_FILE.responseName(operation));

  const requestedPage = request[FIELDS.arguments][FIELDS.page];
  const returnedPage = response[FIELDS.result][FIELDS.page];
  if (typeof requestedPage === "number" && returnedPage !== null) {
    assert(
      returnedPage === requestedPage + 1,
      `${operation} response page must be request page + 1`
    );
  }

  return { request, response };
}

function validateProtocolResultBinding(operation, response, label) {
  const result = response[FIELDS.result];
  assert(result && typeof result === "object", `${label} missing result object`);

  const resultKindChecks = {
    outline: () =>
      Array.isArray(result[FIELDS.entries]) &&
      FIELDS.page in result &&
      !(FIELDS.matches in result),
    read: () =>
      FIELDS.ref in result &&
      FIELDS.content in result &&
      FIELDS.contentType in result &&
      FIELDS.cost in result,
    find: () =>
      Array.isArray(result[FIELDS.matches]) &&
      FIELDS.page in result &&
      !(FIELDS.entries in result),
    info: () =>
      FIELDS.display in result &&
      Array.isArray(result[FIELDS.capabilities]) &&
      !(FIELDS.page in result)
  };

  assert(resultKindChecks[operation]?.(), `${label} result does not match ${operation}`);
}

function validateExampleBudget(operation, request, result) {
  const limit = request[FIELDS.arguments][FIELDS.limitChars];
  if (typeof limit !== "number") {
    return;
  }

  if (operation === OPERATION_NAMES.read) {
    assert(
      charLength(result[FIELDS.content]) <= limit,
      `${operation} content exceeds limit_chars in example`
    );
    return;
  }

  const records =
    operation === OPERATION_NAMES.outline ? result[FIELDS.entries] : result[FIELDS.matches];
  const recordSizes = records.map((record) => ({
    ref: charLength(record[FIELDS.ref]),
    display: charLength(record[FIELDS.display])
  }));
  const totalChars = recordSizes.reduce((sum, record) => sum + record.ref + record.display, 0);
  if (totalChars <= limit) {
    return;
  }

  const oversizedRefRecords = recordSizes.filter((record) => record.ref > limit);
  assert(
    records.length === 1 && oversizedRefRecords.length === 1,
    `${operation} ref + display exceeds limit_chars in example without single oversized ref exception`
  );
  assert(
    recordSizes[0].display > 0 && recordSizes[0].display <= limit,
    `${operation} oversized ref example display must be readable and fit limit_chars`
  );
}

function validateProtocolReadableMappings() {
  for (const operation of OPERATIONS) {
    const { request, response } = validateProtocolPair(operation);
    validateExampleBudget(operation, request, response[FIELDS.result]);

    const readable = readJson(READABLE_EXAMPLE_FILE.result(operation));
    assertDeepEqual(
      readable,
      toReadablePayload(operation, response[FIELDS.result]),
      `${operation} readable JSON must preserve protocol result semantics`
    );

    const mcp = readJson(MCP_EXAMPLE_FILE.response(operation));
    assertDeepEqual(
      mcp[FIELDS.result][FIELDS.structuredContent],
      readable,
      `${operation} MCP structuredContent must match readable JSON example`
    );
  }

  console.log(`protocol/readable mapping ok: ${OPERATIONS.length} operation(s)`);
}

function validateErrorDetails() {
  const errorFiles = listExampleJson(/^error-.*\.json$/);
  for (const errorRelPath of errorFiles) {
    const response = readJson(errorRelPath);
    assert(response[FIELDS.ok] === false, `${errorRelPath} must be an error response`);
    assert(
      !(FIELDS.result in response),
      `${errorRelPath} error response must not include result`
    );
    assert(
      response[FIELDS.operation] === null ||
        OPERATIONS.includes(response[FIELDS.operation]),
      `${errorRelPath} error operation must be known operation or null`
    );
    const details = response[FIELDS.error][FIELDS.details];
    const errorCode = response[FIELDS.error][FIELDS.code];
    const requiredDetails = REQUIRED_ERROR_DETAILS_BY_CODE[errorCode];
    assert(requiredDetails, `${errorRelPath} uses unknown error code ${errorCode}`);
    for (const field of requiredDetails) {
      assert(field in details, `${errorRelPath} missing error.details.${field}`);
    }
  }

  const readableError = readJson(EXAMPLES.readableError);
  assert(
    readableError[FIELDS.code] in REQUIRED_ERROR_DETAILS_BY_CODE,
    "readable-error.json uses unknown error code"
  );
  for (const field of REQUIRED_ERROR_DETAILS_BY_CODE[readableError[FIELDS.code]]) {
    assert(
      field in (readableError[FIELDS.details] ?? {}),
      `readable-error.json missing details.${field}`
    );
  }

  console.log(`error details ok: ${errorFiles.length + 1} file(s)`);
}

function validateManifestSemantics() {
  const manifest = readJson(EXAMPLES.manifest);
  assert(
    manifest[FIELDS.adapter][FIELDS.id] === MARKDOWN_MANIFEST_EXPECTED.adapterId,
    "manifest example must describe docnav-markdown"
  );

  for (const capability of MARKDOWN_MANIFEST_EXPECTED.capabilities) {
    assert(
      manifest[FIELDS.capabilities].includes(capability),
      `markdown manifest example missing capability ${capability}`
    );
  }

  const markdownFormat = manifest[FIELDS.formats].find(
    (format) => format[FIELDS.id] === MARKDOWN_MANIFEST_EXPECTED.formatId
  );
  assert(markdownFormat, "manifest example missing markdown format");
  assert(
    markdownFormat[FIELDS.extensions].includes(MARKDOWN_MANIFEST_EXPECTED.extension),
    "markdown manifest example missing .md extension"
  );
  assert(
    markdownFormat[FIELDS.contentTypes].includes(MARKDOWN_MANIFEST_EXPECTED.contentType),
    "markdown manifest example missing text/markdown content type"
  );

  console.log("manifest semantics ok: markdown capabilities and format");
}

export function validateExampleSemantics() {
  validateProtocolReadableMappings();
  validateErrorDetails();
  validateManifestSemantics();
}
