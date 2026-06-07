import Ajv2020 from "ajv/dist/2020.js";

import { assert, listSchemaJson, readJson } from "./fs-utils.mjs";

export function formatAjvErrors(validate) {
  return (validate.errors ?? [])
    .map((error) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

export function createSchemaAjv() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  for (const schemaRelPath of listSchemaJson()) {
    ajv.addSchema(readJson(schemaRelPath));
  }
  return ajv;
}

export function compileRegisteredSchema(ajv, schemaRelPath) {
  const schema = readJson(schemaRelPath);
  if (!schema.$id) {
    return ajv.compile(schema);
  }
  const validate = ajv.getSchema(schema.$id);
  assert(validate, `registered schema ${schemaRelPath} is not available by $id`);
  return validate;
}
