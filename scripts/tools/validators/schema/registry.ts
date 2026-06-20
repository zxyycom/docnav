import { Ajv2020 } from "ajv/dist/2020.js";
import type { AnySchema, ValidateFunction } from "ajv";

import { isRecord } from "../../type-guards.ts";
import { assert } from "../assertions.ts";
import { listSchemaJson, readJson } from "../json/files.ts";

export function formatAjvErrors(validate: Pick<ValidateFunction, "errors">): string {
  return (validate.errors ?? [])
    .map((error) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

export function createSchemaAjv() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  for (const schemaRelPath of listSchemaJson()) {
    ajv.addSchema(readSchema(schemaRelPath));
  }
  return ajv;
}

export function compileRegisteredSchema(ajv: ReturnType<typeof createSchemaAjv>, schemaRelPath: string): ValidateFunction {
  const schema = readSchema(schemaRelPath);
  const schemaId = isRecord(schema) && typeof schema.$id === "string" ? schema.$id : null;
  if (!schemaId) {
    return ajv.compile(schema);
  }
  const validate = ajv.getSchema(schemaId);
  assert(validate, `registered schema ${schemaRelPath} is not available by $id`);
  return validate;
}

function readSchema(schemaRelPath: string): AnySchema {
  const schema = readJson(schemaRelPath);
  assert(isRecord(schema), `${schemaRelPath} schema root must be an object`);
  return schema as AnySchema;
}
