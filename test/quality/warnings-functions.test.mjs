import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { generateWarnings } from "../../scripts/quality/warnings.mjs";

describe("warnings functions", () => {
  it("generates warnings for complex functions", () => {
    const functions = [
      {
        name: "too_complex",
        file: "crates/docnav/src/complex.rs",
        codeArea: "rust-production",
        startLine: 10,
        endLine: 200,
        lines: 60,
        parameterCount: 8,
        cyclomaticComplexity: { value: 25, source: "lizard" },
        isChanged: true
      }
    ];

    const warnings = generateWarnings({
      files: [],
      functions,
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/complex.rs"] },
      comparisonStatus: "compared"
    });

    assert.ok(warnings.length > 0, "Should generate warnings for complex functions");
    assert.ok(warnings.some((warning) => warning.ruleId === "lizard-cyclomatic-complexity"));
    assert.ok(warnings.some((warning) => warning.ruleId === "lizard-parameter-count"));
  });

  it("uses baseline deltas for changed function warnings", () => {
    const warnings = generateWarnings({
      files: [],
      functions: [{
        name: "too_complex",
        file: "crates/docnav/src/complex.rs",
        codeArea: "rust-production",
        startLine: 10,
        endLine: 90,
        lines: 80,
        parameterCount: 8,
        cyclomaticComplexity: { value: 25, source: "lizard" },
        isChanged: true
      }],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/complex.rs"] },
      baseline: {
        files: [],
        functions: [{
          name: "too_complex",
          file: "crates/docnav/src/complex.rs",
          codeArea: "rust-production",
          startLine: 10,
          endLine: 70,
          lines: 40,
          parameterCount: 4,
          cyclomaticComplexity: { value: 12, source: "lizard" },
          isChanged: false
        }],
        duplicates: []
      },
      comparisonStatus: "compared"
    });

    const ccWarning = warnings.find((warning) => warning.ruleId === "lizard-cyclomatic-complexity");
    assert.ok(ccWarning);
    assert.equal(ccWarning.comparisonBasis, "delta");
    assert.equal(ccWarning.baselineValue, 12);
    assert.equal(ccWarning.deltaValue, 13);
  });

  it("excludes warnings for generated code area", () => {
    const functions = [
      {
        name: "generated_func",
        file: "scripts/generated/code.mjs",
        codeArea: "generated",
        startLine: 1,
        endLine: 100,
        lines: 80,
        parameterCount: 10,
        cyclomaticComplexity: { value: 20, source: "lizard" },
        isChanged: true
      }
    ];

    const warnings = generateWarnings({
      files: [],
      functions,
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["scripts/generated/code.mjs"] },
      comparisonStatus: "compared"
    });

    assert.equal(warnings.length, 0, "Should exclude warnings for generated code area");
  });
});
