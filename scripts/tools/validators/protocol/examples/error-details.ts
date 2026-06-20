import { assert } from "../../assertions.ts";
import {
  EXAMPLES,
  FIELDS,
  OPERATIONS,
  REQUIRED_ERROR_DETAILS_BY_CODE,
} from "../../config.ts";
import { listExampleJson, readJson } from "../../json/files.ts";
import { jsonObject } from "./json.ts";

export function validateErrorDetails() {
  const errorFiles = listExampleJson(/^error-.*\.json$/);
  validateProtocolErrorResponses(errorFiles);
  validateReadableErrorDetails();
  console.log(`error details ok: ${errorFiles.length + 1} file(s)`);
}

function validateProtocolErrorResponses(errorFiles: string[]) {
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
    assertRequiredDetails(errorRelPath, errorCode, details, "error.details");
  }
}

function validateReadableErrorDetails() {
  const readableError = jsonObject(readJson(EXAMPLES.readableError), EXAMPLES.readableError);
  const readableErrorCode = readableError[FIELDS.code];
  assert(typeof readableErrorCode === "string", "readable-error.json code must be a string");
  const details = jsonObject(readableError[FIELDS.details] ?? {}, "readable-error.json details");
  assertRequiredDetails(EXAMPLES.readableError, readableErrorCode, details, "details");
}

function assertRequiredDetails(
  relPath: string,
  errorCode: string,
  details: Record<string, unknown>,
  fieldPrefix: string
) {
  for (const field of requiredDetailsFor(relPath, errorCode)) {
    assert(
      field in details,
      `${relPath} missing ${fieldPrefix}.${field}`,
    );
  }
}

function requiredDetailsFor(relPath: string, errorCode: string): readonly string[] {
  const requiredDetails = (REQUIRED_ERROR_DETAILS_BY_CODE as Record<string, readonly string[]>)[errorCode];
  assert(
    requiredDetails,
    `${relPath} uses unknown error code ${errorCode}`,
  );
  return requiredDetails;
}
