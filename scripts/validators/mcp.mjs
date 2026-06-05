import Ajv2020 from "ajv/dist/2020.js";

import { assert, readJson } from "./fs-utils.mjs";
import {
  FIELDS,
  MCP_EXAMPLE_FILE,
  MCP_STRUCTURED_CONTENT_FORBIDDEN_FIELDS,
  READABLE_SCHEMA_BY_OPERATION
} from "./config.mjs";
import { formatAjvErrors } from "./schema.mjs";

export function validateMcpStructuredContent() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  const cases = Object.entries(READABLE_SCHEMA_BY_OPERATION).map(([operation, schema]) => [
    MCP_EXAMPLE_FILE.response(operation),
    schema
  ]);

  for (const [responseRelPath, schemaRelPath] of cases) {
    const response = readJson(responseRelPath);
    const structuredContent = response?.[FIELDS.result]?.[FIELDS.structuredContent];
    assert(structuredContent && typeof structuredContent === "object", `${responseRelPath} missing structuredContent`);

    for (const field of MCP_STRUCTURED_CONTENT_FORBIDDEN_FIELDS) {
      assert(!(field in structuredContent), `${responseRelPath} leaks ${field} in structuredContent`);
    }

    const validate = ajv.compile(readJson(schemaRelPath));
    if (!validate(structuredContent)) {
      throw new Error(`${responseRelPath} structuredContent failed ${schemaRelPath}: ${formatAjvErrors(validate)}`);
    }
  }
  console.log(`mcp structuredContent ok: ${cases.length} response file(s)`);
}
