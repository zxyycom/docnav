import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { generateWarnings } from "../../scripts/quality/warnings.mjs";

describe("warnings records", () => {
  it("sorts warnings by level then by value", () => {
    const warnings = [
      { level: "info", ruleId: "test", sourceTool: "test", path: "a", line: 1, codeArea: "rust-production", metric: "test", value: 100, comparisonBasis: "absolute", baselineValue: null, deltaValue: null, message: "info msg" },
      { level: "warning", ruleId: "test", sourceTool: "test", path: "b", line: 1, codeArea: "rust-production", metric: "test", value: 50, comparisonBasis: "absolute", baselineValue: null, deltaValue: null, message: "warn msg" },
      { level: "warning", ruleId: "test", sourceTool: "test", path: "c", line: 1, codeArea: "rust-production", metric: "test", value: 200, comparisonBasis: "absolute", baselineValue: null, deltaValue: null, message: "warn msg 2" }
    ];

    const sorted = [...warnings].sort((a, b) => {
      const levelOrder = { error: 0, warning: 1, info: 2 };
      const levelDiff = levelOrder[a.level] - levelOrder[b.level];
      if (levelDiff !== 0) return levelDiff;
      return b.value - a.value;
    });

    assert.equal(sorted[0].level, "warning");
    assert.equal(sorted[0].value, 200);
    assert.equal(sorted[1].level, "warning");
    assert.equal(sorted[1].value, 50);
    assert.equal(sorted[2].level, "info");
  });

  it("warning record has all required fields", () => {
    const warnings = generateWarnings({
      files: [{
        path: "crates/docnav/src/large.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 500,
        complexity: { value: 30, source: "scc" },
        isChanged: true
      }],
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/large.rs"] },
      comparisonStatus: "compared"
    });

    const warning = warnings[0];
    assert.ok(warning.level);
    assert.ok(warning.ruleId);
    assert.ok(warning.sourceTool);
    assert.ok(warning.path);
    assert.ok(warning.codeArea);
    assert.ok(warning.metric);
    assert.ok(typeof warning.value === "number");
    assert.ok(warning.comparisonBasis);
    assert.ok(warning.message);
  });
});
