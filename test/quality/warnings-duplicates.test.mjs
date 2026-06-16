import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { generateWarnings } from "../../scripts/quality/warnings.mjs";

describe("warnings duplicates", () => {
  it("generates duplicate code warnings", () => {
    const duplicates = [
      {
        id: 1,
        tokenCount: 120,
        lineCount: 20,
        locations: [
          { path: "crates/docnav/src/a.rs", startLine: 10, endLine: 30, codeArea: "rust-production" },
          { path: "crates/docnav/src/b.rs", startLine: 50, endLine: 70, codeArea: "rust-production" }
        ],
        codeAreas: ["rust-production"],
        hitsChangedScope: true
      }
    ];

    const warnings = generateWarnings({
      files: [],
      functions: [],
      duplicates,
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: [] },
      comparisonStatus: "compared"
    });

    assert.ok(warnings.some((warning) => warning.ruleId === "pmd-cpd-duplicate-code"));
  });

  it("suppresses duplicate warnings when the same fragment exists in baseline", () => {
    const duplicate = {
      id: 1,
      tokenCount: 120,
      lineCount: 20,
      locations: [
        { path: "crates/docnav/src/a.rs", startLine: 10, endLine: 30, codeArea: "rust-production" },
        { path: "crates/docnav/src/b.rs", startLine: 50, endLine: 70, codeArea: "rust-production" }
      ],
      codeAreas: ["rust-production"],
      hitsChangedScope: true
    };

    const warnings = generateWarnings({
      files: [],
      functions: [],
      duplicates: [duplicate],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/a.rs"] },
      baseline: { files: [], functions: [], duplicates: [duplicate] },
      comparisonStatus: "compared"
    });

    assert.equal(warnings.length, 0);
  });
});
