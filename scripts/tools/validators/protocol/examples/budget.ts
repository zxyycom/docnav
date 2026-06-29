import { assert } from "../../assertions.ts";
import { FIELDS, OPERATION_NAMES } from "../../config.ts";
import { codePointLength, jsonArray, jsonObject } from "./json.ts";

export function validateExampleBudget(
  operation: string,
  request: Record<string, unknown>,
  result: Record<string, unknown>,
): void {
  const requestArgs = jsonObject(request[FIELDS.arguments], `${operation} request arguments`);
  const limit = requestArgs[FIELDS.limit];
  if (typeof limit !== "number") {
    return;
  }

  if (operation === OPERATION_NAMES.read) {
    const content = result[FIELDS.content];
    assert(typeof content === "string", `${operation} content must be a string`);
    assert(
      codePointLength(content) <= limit,
      `${operation} content exceeds limit in example`,
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
    const label = recordObject["label"];
    assert(typeof ref === "string", `${operation} record ${index} ref must be a string`);
    assert(typeof label === "string", `${operation} record ${index} label must be a string`);
    return {
      ref: codePointLength(ref),
      label: codePointLength(label),
    };
  });
  const totalChars = recordSizes.reduce(
    (sum, record) => sum + record.ref + record.label,
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
    `${operation} ref + label exceeds limit in example without single oversized ref exception`,
  );
  assert(
    recordSizes[0].label > 0 && recordSizes[0].label <= limit,
    `${operation} oversized ref example label must be readable and fit limit`,
  );
}
