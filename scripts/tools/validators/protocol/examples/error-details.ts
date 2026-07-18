import { assert } from "../../assertions.ts";
import { FIELDS, OPERATIONS } from "../../config.ts";
import { listExampleJson, readJson } from "../../json/files.ts";
import { jsonObject } from "./json.ts";

export function validateErrorDetails() {
  const errorFiles = listExampleJson(/^error-.*\.json$/);
  validateProtocolErrorResponses(errorFiles);
  console.log(`error details ok: ${errorFiles.length} file(s)`);
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
    assert(
      typeof error.owner === "string" && error.owner.length > 0,
      `${errorRelPath} error owner must be a non-empty string`,
    );
    assert(
      Array.isArray(error.guidance) && error.guidance.length > 0,
      `${errorRelPath} error guidance must be a non-empty array`,
    );
    assert(Object.keys(details).length > 0, `${errorRelPath} error details must not be empty`);
  }
}
