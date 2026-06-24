import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../model/config.ts";
import type { FileMetric, FunctionMetric } from "../../model/schema.ts";
import { generateWarningChannels } from "./generator.ts";

// @case AUX-QUALITY-WARNINGS-001
describe("quality warning generation", () => {
  it("uses scc code lines and low decision-token allowance for file-size warnings", () => {
    const files = [
      qualityFile("scripts/comment-heavy.ts", { lines: 420, codeLines: 120 }),
      qualityFile("scripts/low-token-config.ts", {
        lines: 540,
        codeLines: 500,
        decisionTokens: 10
      }),
      qualityFile("scripts/high-token-module.ts", {
        lines: 540,
        codeLines: 500,
        decisionTokens: 11
      })
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
      [["scc-file-code-lines", "scripts/high-token-module.ts", "code-lines", 500]]
    );
    assert.match(warnings.all[0]!.message, /500 code lines/);
    assert.match(warnings.all[0]!.suggestion ?? "", /responsibility/);
    assert.doesNotMatch(warnings.all[0]!.suggestion ?? "", /\bsplitting\b|\bsplit\b/i);
  });

  it("uses complexity-aware function code density thresholds", () => {
    const functions = [
      qualityFunction("simpleLongEnough.ts", { complexity: 4, lines: 120 }),
      qualityFunction("simpleTooLong.ts", { complexity: 4, lines: 151 }),
      qualityFunction("normalTooLong.ts", { complexity: 5, lines: 51 })
    ];

    const warnings = generateWarningChannels({
      baseline: null,
      comparisonStatus: "baseline-unavailable",
      config: DEFAULT_CONFIG,
      duplicates: [],
      files: [],
      functions,
      scope: { changed: false, changedFiles: [] }
    });

    assert.deepEqual(
      warnings.all.map((warning) => [warning.ruleId, warning.path, warning.metric, warning.value]),
      [
        ["lizard-function-code-density", "simpleTooLong.ts", "function-code-density", 151],
        ["lizard-function-code-density", "normalTooLong.ts", "function-code-density", 51]
      ]
    );
    assert.match(warnings.all[0]!.message, /151 code lines at cyclomatic complexity 4/);
    assert.match(warnings.all[0]!.message, /threshold: 150 code lines for CC < 5/);
    assert.match(warnings.all[1]!.message, /threshold: 50 code lines/);
  });
});

function qualityFile(
  path: string,
  options: { codeLines: number; decisionTokens?: number; lines: number }
): FileMetric {
  return {
    path,
    language: "TypeScript",
    codeArea: "typescript-production-scripts",
    lines: options.lines,
    codeLines: options.codeLines,
    decisionTokens: { value: options.decisionTokens ?? 1, source: "scc" },
    isChanged: false
  };
}

function qualityFunction(
  path: string,
  options: { complexity: number; lines: number }
): FunctionMetric {
  return {
    file: path,
    name: "example",
    codeArea: "typescript-production-scripts",
    startLine: 1,
    endLine: options.lines,
    lines: options.lines,
    parameterCount: 1,
    cyclomaticComplexity: { value: options.complexity, source: "lizard" },
    isChanged: false
  };
}
