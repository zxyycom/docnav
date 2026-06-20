import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../model/config.ts";
import type { FileMetric } from "../../model/schema.ts";
import { generateWarningChannels } from "./index.ts";

// @case AUX-QUALITY-WARNINGS-001
describe("quality warning generation", () => {
  it("uses scc code lines instead of physical lines for file-size warnings", () => {
    const files = [
      qualityFile("scripts/comment-heavy.ts", { lines: 420, codeLines: 120 }),
      qualityFile("scripts/large-code.ts", { lines: 420, codeLines: 360 })
    ];

    const warnings = generateWarningChannels({
      baseline: null,
      comparisonStatus: "baseline-unavailable",
      config: DEFAULT_CONFIG,
      duplicates: [],
      files,
      functions: [],
      scope: { changed: false, changedFiles: [] }
    });

    assert.deepEqual(
      warnings.all.map((warning) => [warning.ruleId, warning.path, warning.metric, warning.value]),
      [["scc-file-code-lines", "scripts/large-code.ts", "code-lines", 360]]
    );
    assert.match(warnings.all[0]!.message, /360 code lines/);
  });
});

function qualityFile(
  path: string,
  options: { codeLines: number; lines: number }
): FileMetric {
  return {
    path,
    language: "TypeScript",
    codeArea: "node-production-scripts",
    lines: options.lines,
    codeLines: options.codeLines,
    complexity: { value: 1, source: "scc" },
    isChanged: false
  };
}
