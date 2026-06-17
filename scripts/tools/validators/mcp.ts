import { assert, readJson } from "./fs-utils.ts";
import {
  FIELDS,
  MCP_EXAMPLE_FILE,
  MCP_STRUCTURED_CONTENT_FORBIDDEN_FIELDS,
  READABLE_SCHEMA_BY_OPERATION
} from "./config.ts";
import { compileRegisteredSchema, createSchemaAjv, formatAjvErrors } from "./schema-registry.ts";

export function validateMcpStructuredContent() {
  const ajv = createSchemaAjv();
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

    const validate = compileRegisteredSchema(ajv, schemaRelPath);
    if (!validate(structuredContent)) {
      throw new Error(`${responseRelPath} structuredContent failed ${schemaRelPath}: ${formatAjvErrors(validate)}`);
    }
  }
  console.log(`mcp structuredContent ok: ${cases.length} response file(s)`);
}
