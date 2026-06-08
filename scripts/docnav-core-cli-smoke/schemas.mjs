import { schemaPaths } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { expect } from "./assertions.mjs";
import {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "../validators/schema-registry.mjs";

export function compileSchemas() {
  const ajv = createSchemaAjv();
  return Object.fromEntries(
    Object.entries(schemaPaths).map(([name, relPath]) => [
      name,
      compileRegisteredSchema(ajv, relPath)
    ])
  );
}

export function validateSchema(record, name, value) {
  const validate = smokeState.validators[name];
  expect(record, Boolean(validate), `schema validator exists for ${name}`);
  const ok = validate(value);
  const details = ok ? "" : `: ${formatAjvErrors(validate)}`;
  expect(record, ok, `${name} schema valid${details}`);
}

