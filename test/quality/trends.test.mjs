import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { createEmptyMetrics } from "../../scripts/quality/schema.mjs";
import { buildAggregates, generateTrends } from "../../scripts/quality/scan.mjs";

describe("trend generation", () => {
  it("generates baseline deltas for file, function, language, code-area, and duplicate metrics", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    metrics.fileMetrics = [
      {
        path: "crates/docnav/src/lib.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 120,
        codeLines: 90,
        complexity: { value: 20, source: "scc" },
        isChanged: true
      }
    ];
    metrics.functionMetrics = [
      {
        name: "parse",
        file: "crates/docnav/src/lib.rs",
        codeArea: "rust-production",
        startLine: 10,
        endLine: 60,
        lines: 50,
        parameterCount: 4,
        cyclomaticComplexity: { value: 12, source: "lizard" },
        isChanged: true
      }
    ];
    metrics.duplicateCode = [
      {
        id: 1,
        tokenCount: 90,
        lineCount: 12,
        locations: [
          { path: "crates/docnav/src/a.rs", startLine: 1, endLine: 12, codeArea: "rust-production" },
          { path: "crates/docnav/src/b.rs", startLine: 20, endLine: 32, codeArea: "rust-production" }
        ],
        codeAreas: ["rust-production"],
        hitsChangedScope: true
      }
    ];
    metrics.aggregates = buildAggregates({
      fileMetrics: metrics.fileMetrics,
      functionMetrics: metrics.functionMetrics,
      duplicateCode: metrics.duplicateCode,
      byLanguage: [
        { language: "Rust", files: 1, lines: 120, codeLines: 90, commentLines: 20, blankLines: 10, complexitySource: "scc" }
      ],
      config: DEFAULT_CONFIG
    });

    const baselineSnapshot = {
      fingerprints: {},
      fileMetrics: [],
      functionMetrics: [],
      duplicateCode: [],
      aggregates: buildAggregates({
        fileMetrics: [{
          path: "crates/docnav/src/lib.rs",
          language: "Rust",
          codeArea: "rust-production",
          lines: 100,
          codeLines: 75,
          complexity: { value: 10, source: "scc" },
          isChanged: false
        }],
        functionMetrics: [{
          name: "parse",
          file: "crates/docnav/src/lib.rs",
          codeArea: "rust-production",
          startLine: 10,
          endLine: 40,
          lines: 30,
          parameterCount: 2,
          cyclomaticComplexity: { value: 5, source: "lizard" },
          isChanged: false
        }],
        duplicateCode: [],
        byLanguage: [
          { language: "Rust", files: 1, lines: 100, codeLines: 75, commentLines: 15, blankLines: 10, complexitySource: "scc" }
        ],
        config: DEFAULT_CONFIG
      })
    };

    const trends = generateTrends(metrics, baselineSnapshot);
    const trendMap = new Map(trends.map((trend) => [trend.metric, trend]));

    assert.equal(trendMap.get("total-file-complexity").delta, 10);
    assert.equal(trendMap.get("total-function-lines").delta, 20);
    assert.equal(trendMap.get("total-function-parameters").delta, 2);
    assert.equal(trendMap.get("total-function-cyclomatic-complexity").delta, 7);
    assert.equal(trendMap.get("duplicate-fragments").baseline, 0);
    assert.equal(trendMap.get("duplicate-fragments").delta, 1);
    assert.ok(trendMap.has("lang-Rust-share"));
    assert.equal(trendMap.get("area-rust-production-function-lines").delta, 20);
  });
});
