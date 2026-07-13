import { describe, expect, test } from "bun:test";

import { parseWarningsNdjson, selectAnnotationWarnings } from "./warnings.ts";

describe("quality warning annotations", () => {
  test("keeps accepted warnings in machine records but selects only unaccepted warnings", () => {
    const content = [
      JSON.stringify({
        acceptedReason: "known baseline",
        level: "warning",
        message: "accepted warning",
        path: "src/accepted.ts",
        ruleId: "accepted-rule"
      }),
      JSON.stringify({
        level: "warning",
        message: "actionable warning",
        path: "src/actionable.ts",
        ruleId: "actionable-rule"
      }),
      JSON.stringify({
        level: "info",
        message: "informational record",
        path: "src/info.ts",
        ruleId: "info-rule"
      })
    ].join("\n");

    const parsed = parseWarningsNdjson(content);

    expect(parsed.diagnostics).toEqual([]);
    expect(parsed.warnings[0]?.acceptedReason).toBe("known baseline");
    expect(selectAnnotationWarnings(parsed.warnings).map((warning) => warning.ruleId)).toEqual([
      "actionable-rule"
    ]);
  });
});
