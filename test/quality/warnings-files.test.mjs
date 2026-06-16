import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { generateWarnings } from "../../scripts/quality/warnings.mjs";

describe("warnings files", () => {
  it("generates warnings for files exceeding thresholds", () => {
    const files = [
      {
        path: "crates/docnav/src/large.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 500,
        complexity: { value: 30, source: "scc" },
        isChanged: true
      }
    ];

    const warnings = generateWarnings({
      files,
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/large.rs"] },
      comparisonStatus: "compared"
    });

    assert.ok(warnings.length > 0, "Should generate warnings for large files");
    assert.ok(warnings.some((warning) => warning.ruleId === "scc-file-lines"));
  });

  it("uses baseline deltas for changed file warnings", () => {
    const warnings = generateWarnings({
      files: [{
        path: "crates/docnav/src/large.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 500,
        complexity: { value: 35, source: "scc" },
        isChanged: true
      }],
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/large.rs"] },
      baseline: {
        files: [{
          path: "crates/docnav/src/large.rs",
          language: "Rust",
          codeArea: "rust-production",
          lines: 350,
          complexity: { value: 20, source: "scc" },
          isChanged: false
        }],
        functions: [],
        duplicates: []
      },
      comparisonStatus: "compared"
    });

    const lineWarning = warnings.find((warning) => warning.ruleId === "scc-file-lines");
    assert.ok(lineWarning);
    assert.equal(lineWarning.comparisonBasis, "delta");
    assert.equal(lineWarning.baselineValue, 350);
    assert.equal(lineWarning.deltaValue, 150);
  });

  it("does not warn for changed files when baseline delta is below threshold", () => {
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
      baseline: {
        files: [{
          path: "crates/docnav/src/large.rs",
          language: "Rust",
          codeArea: "rust-production",
          lines: 450,
          complexity: { value: 25, source: "scc" },
          isChanged: false
        }],
        functions: [],
        duplicates: []
      },
      comparisonStatus: "compared"
    });

    assert.equal(warnings.length, 0);
  });

  it("does not warn for unchanged historical hotspots", () => {
    const warnings = generateWarnings({
      files: [{
        path: "crates/docnav/src/hotspot.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 900,
        complexity: { value: 100, source: "scc" },
        isChanged: false
      }],
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/other.rs"] },
      baseline: {
        files: [{
          path: "crates/docnav/src/hotspot.rs",
          language: "Rust",
          codeArea: "rust-production",
          lines: 900,
          complexity: { value: 100, source: "scc" },
          isChanged: false
        }],
        functions: [],
        duplicates: []
      },
      comparisonStatus: "compared"
    });

    assert.equal(warnings.length, 0);
  });

  it("generates no warnings when comparison is input-unchanged", () => {
    const files = [
      {
        path: "crates/docnav/src/large.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 500,
        complexity: { value: 30, source: "scc" },
        isChanged: false
      }
    ];

    const warnings = generateWarnings({
      files,
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: false, changedFiles: [] },
      comparisonStatus: "input-unchanged"
    });

    assert.equal(warnings.length, 0, "Should not generate warnings for input-unchanged");
  });

  it("respects watchlist-only policy for fixtures", () => {
    const files = [
      {
        path: "test/fixtures/large.rs",
        language: "Rust",
        codeArea: "fixtures-examples",
        lines: 500,
        complexity: { value: 30, source: "scc" },
        isChanged: true
      }
    ];

    const warnings = generateWarnings({
      files,
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["test/fixtures/large.rs"] },
      comparisonStatus: "compared"
    });

    for (const warning of warnings) {
      assert.equal(
        warning.level,
        "info",
        `Fixture warning should be info, got ${warning.level}: ${warning.message}`
      );
    }
  });

  it("does not generate warnings below absolute floor", () => {
    const files = [
      {
        path: "crates/docnav/src/small.rs",
        language: "Rust",
        codeArea: "rust-production",
        lines: 50,
        complexity: { value: 5, source: "scc" },
        isChanged: true
      }
    ];

    const warnings = generateWarnings({
      files,
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["crates/docnav/src/small.rs"] },
      comparisonStatus: "compared"
    });

    assert.equal(warnings.length, 0, "Should not generate warnings for values below absolute floor");
  });
});
