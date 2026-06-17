import Ajv2020Module from "ajv/dist/2020.js";

import { assert, listSchemaJson, readJson } from "./fs-utils.ts";

export function formatAjvErrors(validate: any) {
  return (validate.errors ?? [])
    .map((error: any) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

export function createSchemaAjv() {
  const Ajv2020 = (Ajv2020Module as any).default ?? Ajv2020Module;
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  for (const schemaRelPath of listSchemaJson()) {
    ajv.addSchema(readJson(schemaRelPath));
  }
  return ajv;
}

export function compileRegisteredSchema(ajv: any, schemaRelPath: any) {
  const schema = readJson(schemaRelPath);
  if (!schema.$id) {
    return ajv.compile(schema);
  }
  const validate = ajv.getSchema(schema.$id);
  assert(validate, `registered schema ${schemaRelPath} is not available by $id`);
  return validate;
}
