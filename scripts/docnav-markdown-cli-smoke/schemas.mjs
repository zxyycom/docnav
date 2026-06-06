import fs from "node:fs";
import path from "node:path";

import Ajv2020 from "ajv/dist/2020.js";

import { root, schemaPaths } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { expect } from "./assertions.mjs";

export function compileSchemas() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  return Object.fromEntries(
    Object.entries(schemaPaths).map(([name, relPath]) => [
      name,
      ajv.compile(JSON.parse(fs.readFileSync(path.join(root, relPath), "utf8")))
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

function formatAjvErrors(validate) {
  return (validate.errors ?? [])
    .map((error) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

