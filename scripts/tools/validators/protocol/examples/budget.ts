import { assert } from "../../assertions.ts";
import { FIELDS, OPERATION_NAMES } from "../../config.ts";
import { codePointLength, jsonArray, jsonObject } from "./json.ts";

export function validateExampleBudget(
  operation: string,
  request: Record<string, unknown>,
  result: Record<string, unknown>,
): void {
  const requestArgs = jsonObject(request[FIELDS.arguments], `${operation} request arguments`);
  const limit = requestArgs[FIELDS.limitChars];
  if (typeof limit !== "number") {
    return;
  }

  if (operation === OPERATION_NAMES.read) {
    const content = result[FIELDS.content];
    assert(typeof content === "string", `${operation} content must be a string`);
    assert(
      codePointLength(content) <= limit,
      `${operation} content exceeds limit_chars in example`,
    );
    return;
  }

  const records =
    operation === OPERATION_NAMES.outline
      ? jsonArray(result[FIELDS.entries], `${operation} entries`)
      : jsonArray(result[FIELDS.matches], `${operation} matches`);
  const recordSizes = records.map((record, index) => {
    const recordObject = jsonObject(record, `${operation} record ${index}`);
    const ref = recordObject[FIELDS.ref];
    const display = recordObject[FIELDS.display];
    assert(typeof ref === "string", `${operation} record ${index} ref must be a string`);
    assert(typeof display === "string", `${operation} record ${index} display must be a string`);
    return {
      ref: codePointLength(ref),
      display: codePointLength(display),
    };
  });
  const totalChars = recordSizes.reduce(
    (sum, record) => sum + record.ref + record.display,
    0,
  );
  if (totalChars <= limit) {
    return;
  }

  const oversizedRefRecords = recordSizes.filter(
    (record) => record.ref > limit,
  );
  assert(
    records.length === 1 && oversizedRefRecords.length === 1,
    `${operation} ref + display exceeds limit_chars in example without single oversized ref exception`,
  );
  assert(
    recordSizes[0].display > 0 && recordSizes[0].display <= limit,
    `${operation} oversized ref example display must be readable and fit limit_chars`,
  );
}
