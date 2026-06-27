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
  const errorSchema = protocolErrorSchema(defs, label);
  const protocolCodes = protocolErrorCodes(errorSchema, label);
  const protocolCodeSet = new Set(protocolCodes);
  const branches = jsonArray(errorSchema.allOf, `${label} error.allOf`);
  const requirements: Record<string, readonly string[]> = {};

  for (const [index, branchValue] of branches.entries()) {
    const branchLabel = `${label} error.allOf[${index}]`;
    const branch = jsonObject(branchValue, branchLabel);
    const code = branchCode(branch, branchLabel);
    assert(protocolCodeSet.has(code), `${label} has error details rule for unknown code ${code}`);
    assert(!(code in requirements), `${label} duplicates error details rule for ${code}`);

    requirements[code] = Object.freeze(
      requiredDetailsFromSchema(
        defs,
        branchDetailsSchema(branch, branchLabel),
        `${branchLabel}.then.properties.details`
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

function protocolErrorSchema(defs: Record<string, unknown>, label: string): Record<string, unknown> {
  const failure = jsonObject(defs.failure, `${label} $defs.failure`);
  const failureProperties = jsonObject(
    failure.properties,
    `${label} $defs.failure.properties`
  );
  return jsonObject(
    failureProperties.error,
    `${label} $defs.failure.properties.error`
  );
}

function protocolErrorCodes(
  errorSchema: Record<string, unknown>,
  label: string
): readonly string[] {
  const errorProperties = jsonObject(
    errorSchema.properties,
    `${label} error.properties`
  );
  const codeSchema = jsonObject(errorProperties.code, `${label} error.properties.code`);
  return jsonStringArray(codeSchema.enum, `${label} error code enum`);
}

function branchCode(branch: Record<string, unknown>, branchLabel: string): string {
  const condition = jsonObject(branch["if"], `${branchLabel}.if`);
  const conditionProperties = jsonObject(
    condition.properties,
    `${branchLabel}.if.properties`
  );
  const conditionCode = jsonObject(
    conditionProperties.code,
    `${branchLabel}.if.properties.code`
  );
  return jsonString(
    conditionCode.const,
    `${branchLabel}.if.properties.code.const`
  );
}

function branchDetailsSchema(
  branch: Record<string, unknown>,
  branchLabel: string
): unknown {
  const projection = jsonObject(branch.then, `${branchLabel}.then`);
  const projectionProperties = jsonObject(
    projection.properties,
    `${branchLabel}.then.properties`
  );
  return projectionProperties.details;
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
