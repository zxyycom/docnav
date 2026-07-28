import { describe, expect, test } from "bun:test";

import {
  classifyFiles,
  generateWarningChannels,
  validateMetrics
} from "../src/index.ts";
import { TEST_QUALITY_CONFIG as config } from "./config.ts";

describe("script quality core", () => {
  test("classifies files using caller-provided code areas", () => {
    const fileMap = classifyFiles(["scripts/a.ts", "scripts/a.test.ts"], config.codeAreas, config.generatedFiles);

    expect(fileMap.get("typescript-production-scripts")).toEqual(["scripts/a.ts"]);
  });

  test("rejects a metrics envelope without metadata", () => {
    const validation = validateMetrics({});

    expect(validation.valid).toBe(false);
    expect(validation.errors.includes("metrics.metadata is required")).toBe(true);
  });

  test("generates warning channels from caller-provided thresholds", () => {
    const callerConfig = {
      ...config,
      scc: {
        fileCodeLines: {
          absoluteFloor: 10,
          changedDelta: 2,
          lowDecisionTokenAllowance: {
            codeLineFloor: 20,
            maxDecisionTokens: 5
          }
        }
      }
    };
    const warnings = generateWarningChannels({
      files: [
        {
          codeArea: "typescript-production-scripts",
          codeLines: 11,
          decisionTokens: { source: "scc", value: 6 },
          isChanged: true,
          language: "TypeScript",
          lines: 12,
          path: "scripts/a.ts"
        }
      ],
      functions: [],
      duplicates: [],
      config: callerConfig,
      scope: { changed: true, changedFiles: ["scripts/a.ts"] },
      baseline: null,
      comparisonStatus: "baseline-unavailable",
      validateAcceptedWarnings: false
    });

    expect(warnings.all.map((warning) => [
      warning.ruleId,
      warning.codeArea,
      warning.path,
      warning.value
    ])).toEqual([[
      "scc-file-code-lines",
      "typescript-production-scripts",
      "scripts/a.ts",
      11
    ]]);
    expect(warnings.changed).toHaveLength(0);
  });
});
