import { assert } from "../assertions.ts";
import { SCHEMAS } from "../config.ts";
import { readJson } from "../json/files.ts";
import { isRecord, isStringArray } from "../../type-guards.ts";

export type RequiredErrorDetailsByCode = Readonly<Record<string, readonly string[]>>;

export function loadProtocolErrorDetailsRequirements(): RequiredErrorDetailsByCode {
  const schema = readJson(SCHEMAS.protocolResponse);
  return protocolErrorDetailsRequirements(schema, SCHEMAS.protocolResponse);
}

function protocolErrorDetailsRequirements(
  schema: unknown,
  label: string
): RequiredErrorDetailsByCode {
  const root = jsonObject(schema, label);
  const defs = jsonObject(root.$defs, `${label} $defs`);
  const failure = jsonObject(defs.failure, `${label} $defs.failure`);
  const failureProperties = jsonObject(
    failure.properties,
    `${label} $defs.failure.properties`
  );
  const errorSchema = jsonObject(
    failureProperties.error,
    `${label} $defs.failure.properties.error`
  );
  const errorProperties = jsonObject(
    errorSchema.properties,
    `${label} error.properties`
  );
  const codeSchema = jsonObject(errorProperties.code, `${label} error.properties.code`);
  const protocolCodes = jsonStringArray(codeSchema.enum, `${label} error code enum`);
  const protocolCodeSet = new Set(protocolCodes);
  const branches = jsonArray(errorSchema.allOf, `${label} error.allOf`);
  const requirements: Record<string, readonly string[]> = {};

  for (const [index, branchValue] of branches.entries()) {
    const branch = jsonObject(branchValue, `${label} error.allOf[${index}]`);
    const condition = jsonObject(branch["if"], `${label} error.allOf[${index}].if`);
    const conditionProperties = jsonObject(
      condition.properties,
      `${label} error.allOf[${index}].if.properties`
    );
    const conditionCode = jsonObject(
      conditionProperties.code,
      `${label} error.allOf[${index}].if.properties.code`
    );
    const code = jsonString(
      conditionCode.const,
      `${label} error.allOf[${index}].if.properties.code.const`
    );
    assert(protocolCodeSet.has(code), `${label} has error details rule for unknown code ${code}`);
    assert(!(code in requirements), `${label} duplicates error details rule for ${code}`);

    const projection = jsonObject(branch.then, `${label} error.allOf[${index}].then`);
    const projectionProperties = jsonObject(
      projection.properties,
      `${label} error.allOf[${index}].then.properties`
    );
    requirements[code] = Object.freeze(
      requiredDetailsFromSchema(
        defs,
        projectionProperties.details,
        `${label} error.allOf[${index}].then.properties.details`
      )
    );
  }

  for (const code of protocolCodes) {
    assert(code in requirements, `${label} missing error details rule for ${code}`);
  }
  assert(
    Object.keys(requirements).length === protocolCodes.length,
    `${label} error details rules must match code enum`
  );

  return Object.freeze(requirements);
}

function requiredDetailsFromSchema(
  defs: Record<string, unknown>,
  schema: unknown,
  label: string
): string[] {
  const detailsSchema = jsonObject(schema, label);
  if ("$ref" in detailsSchema) {
    const ref = jsonString(detailsSchema.$ref, `${label}.$ref`);
    const defPrefix = "#/$defs/";
    assert(ref.startsWith(defPrefix), `${label}.$ref must target local $defs`);
    const defName = ref.slice(defPrefix.length);
    const def = jsonObject(defs[defName], `${label} ${ref}`);
    return jsonStringArray(def.required, `${label} ${ref}.required`);
  }

  return jsonStringArray(detailsSchema.required, `${label}.required`);
}

function jsonObject(value: unknown, label: string): Record<string, unknown> {
  assert(isRecord(value) && !Array.isArray(value), `${label} must be an object`);
  return value;
}

function jsonArray(value: unknown, label: string): unknown[] {
  assert(Array.isArray(value), `${label} must be an array`);
  return value;
}

function jsonString(value: unknown, label: string): string {
  assert(typeof value === "string", `${label} must be a string`);
  return value;
}

function jsonStringArray(value: unknown, label: string): string[] {
  assert(isStringArray(value), `${label} must be a string array`);
  return value;
}
