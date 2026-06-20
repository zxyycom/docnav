import {
  listExampleJson,
  listSchemaJson,
  readJson
} from "../json/files.ts";
import { walk } from "../repo/files.ts";
import { toAbs, toRel } from "../repo/paths.ts";
import { assert } from "../assertions.ts";
import {
  EXAMPLES,
  FIELDS,
  FILE_SYSTEM,
  OPERATION_NAMES,
  REQUIRED_ERROR_DETAILS_BY_CODE,
  SCHEMAS
} from "../config.ts";
import {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "./registry.ts";
import { isRecord } from "../../type-guards.ts";

export {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "./registry.ts";

function validateWithSchema(ajv: ReturnType<typeof createSchemaAjv>, schemaRelPath: string, dataRelPaths: string[]): void {
  const validate = compileRegisteredSchema(ajv, schemaRelPath);
  for (const dataRelPath of dataRelPaths) {
    const data = readJson(dataRelPath);
    if (!validate(data)) {
      throw new Error(`${dataRelPath} failed ${schemaRelPath}: ${formatAjvErrors(validate)}`);
    }
  }
  console.log(`schema ok: ${schemaRelPath} (${dataRelPaths.length} file(s))`);
}

function validateStrictSchemaCompilation() {
  const schemaRelPaths = listSchemaJson();
  const expectedSchemas = Object.values(SCHEMAS);

  for (const expected of expectedSchemas) {
    assert(schemaRelPaths.includes(expected), `missing expected schema ${expected}`);
  }

  const ajv = createSchemaAjv();
  for (const schemaRelPath of schemaRelPaths) {
    compileRegisteredSchema(ajv, schemaRelPath);
  }
  console.log(`schema strict compile ok: ${schemaRelPaths.length} schema file(s)`);
}

function validateProtocolResponseBindingSchema() {
  const ajv = createSchemaAjv();
  const validate = compileRegisteredSchema(ajv, SCHEMAS.protocolResponse);
  const mismatched = structuredClone(readJson(EXAMPLES.protocolReadResponse));
  assert(isRecord(mismatched), "protocol read response example must be an object");
  mismatched[FIELDS.operation] = OPERATION_NAMES.outline;

  assert(
    !validate(mismatched),
    "protocol response schema must reject operation/result mismatches"
  );
  console.log("protocol response operation/result binding ok");
}

function validateProtocolResponseErrorDetailsSchema() {
  const ajv = createSchemaAjv();
  const validate = compileRegisteredSchema(ajv, SCHEMAS.protocolResponse);

  for (const [code, requiredDetails] of Object.entries(REQUIRED_ERROR_DETAILS_BY_CODE)) {
    const validResponse = protocolErrorResponse(
      code,
      Object.fromEntries(requiredDetails.map((field) => [field, "test"]))
    );
    assert(
      validate(validResponse),
      `protocol response schema must accept ${code} with required details: ${formatAjvErrors(validate)}`
    );

    const missingFirstRequiredDetail = protocolErrorResponse(
      code,
      Object.fromEntries(requiredDetails.slice(1).map((field) => [field, "test"]))
    );
    assert(
      !validate(missingFirstRequiredDetail),
      `protocol response schema must reject ${code} without error.details.${requiredDetails[0]}`
    );
  }

  console.log("protocol response error details requirements ok");
}

function protocolErrorResponse(code: string, details: Record<string, string>) {
  return {
    protocol_version: "0.1",
    request_id: "req-error-details",
    operation: null,
    ok: false,
    error: {
      code,
      message: "test error",
      details
    }
  };
}

export function validateJsonSyntax() {
  const jsonFiles = walk(toAbs(FILE_SYSTEM.docsDir), (filePath) =>
    filePath.endsWith(FILE_SYSTEM.jsonExtension)
  );
  for (const filePath of jsonFiles) {
    readJson(toRel(filePath));
  }
  console.log(`json syntax ok: ${jsonFiles.length} file(s)`);
}

export function validateSchemas() {
  validateStrictSchemaCompilation();
  const ajv = createSchemaAjv();
  const cases = [
    {
      schema: SCHEMAS.protocolRequest,
      data: listExampleJson(/^protocol-.*-request\.json$/)
    },
    {
      schema: SCHEMAS.protocolResponse,
      data: [
        ...listExampleJson(/^protocol-.*-response\.json$/),
        ...listExampleJson(/^error-.*\.json$/)
      ]
    },
    {
      schema: SCHEMAS.manifest,
      data: [EXAMPLES.manifest]
    },
    {
      schema: SCHEMAS.probeResult,
      data: [EXAMPLES.probeResult]
    },
    {
      schema: SCHEMAS.readableOutline,
      data: [EXAMPLES.readableOutline]
    },
    {
      schema: SCHEMAS.readableRead,
      data: [EXAMPLES.readableRead]
    },
    {
      schema: SCHEMAS.readableFind,
      data: [EXAMPLES.readableFind]
    },
    {
      schema: SCHEMAS.readableInfo,
      data: [EXAMPLES.readableInfo]
    },
    {
      schema: SCHEMAS.readableError,
      data: [EXAMPLES.readableError]
    }
  ];

  for (const item of cases) {
    validateWithSchema(ajv, item.schema, item.data);
  }
  validateProtocolResponseBindingSchema();
  validateProtocolResponseErrorDetailsSchema();
}
