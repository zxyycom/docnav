import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import {
  BASELINE_STATUSES,
  COMPARISON_STATUSES,
  createEmptyMetrics
} from "../../scripts/quality/schema.mjs";
import { configureBaseline, parseArgs } from "../../scripts/quality/scan-cli.mjs";
import { generateWarnings } from "../../scripts/quality/warnings.mjs";
import { detectTextOnlyChange, locateBaselineCommit } from "../../scripts/quality/baseline.mjs";
import { runGit } from "./helpers.mjs";

describe("baseline and comparison status", () => {
  it("BASELINE_STATUSES values match spec", () => {
    const statuses = [...BASELINE_STATUSES].sort();
    const expected = [
      "baseline-materialization-failed",
      "baseline-scan-failed",
      "baseline-skipped",
      "generated",
      "history-unavailable",
      "no-baseline-commit"
    ].sort();
    assert.deepEqual(statuses, expected);
  });

  it("COMPARISON_STATUSES values match spec", () => {
    const statuses = [...COMPARISON_STATUSES].sort();
    const expected = ["baseline-unavailable", "compared", "input-unchanged"].sort();
    assert.deepEqual(statuses, expected);
  });

  it("metrics object correctly stores baseline status", () => {
    const metrics = createEmptyMetrics({
      repository: "/test",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    metrics.baseline.status = "generated";
    metrics.baseline.commitSha = "def456";
    metrics.baseline.commitDate = "2025-01-01T00:00:00Z";
    metrics.comparisonStatus = "compared";

    assert.equal(metrics.baseline.status, "generated");
    assert.equal(metrics.comparisonStatus, "compared");
  });

  it("metrics object handles baseline-unavailable", () => {
    const metrics = createEmptyMetrics({
      repository: "/test",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    metrics.baseline.status = "history-unavailable";
    metrics.comparisonStatus = "baseline-unavailable";

    assert.equal(metrics.baseline.status, "history-unavailable");
    assert.equal(metrics.comparisonStatus, "baseline-unavailable");
  });

  it("scan args enable baseline by default and skip only when explicit", () => {
    assert.equal(parseArgs([]).skipBaseline, false);
    assert.equal(parseArgs(["--skip-baseline"]).skipBaseline, true);
  });

  it("configureBaseline records an explicit skipped status for --skip-baseline", () => {
    const metrics = createEmptyMetrics({
      repository: "/test",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    configureBaseline({
      metrics,
      opts: { baseline: null, skipBaseline: true },
      tools: [],
      fatalIssues: [],
      root: "/test"
    });

    assert.equal(metrics.baseline.status, "baseline-skipped");
    assert.equal(metrics.baseline.commitSha, null);
    assert.equal(metrics.baseline.metadata, null);
  });

  it("metrics object handles input-unchanged", () => {
    const metrics = createEmptyMetrics({
      repository: "/test",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    metrics.baseline.status = "generated";
    metrics.comparisonStatus = "input-unchanged";

    assert.equal(metrics.comparisonStatus, "input-unchanged");
  });

  it("warnings are empty when comparison is baseline-unavailable", () => {
    const warnings = generateWarnings({
      files: [{ path: "a.rs", language: "Rust", codeArea: "rust-production", lines: 500, complexity: { value: 30, source: "scc" }, isChanged: true }],
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["a.rs"] },
      comparisonStatus: "baseline-unavailable"
    });

    assert.equal(warnings.length, 0, "No warnings should be generated when baseline is unavailable");
  });

  it("locateBaselineCommit reports no-baseline-commit for a single-commit repository", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-baseline-"));
    try {
      runGit(tempDir, ["init"]);
      runGit(tempDir, ["config", "user.email", "docnav@example.test"]);
      runGit(tempDir, ["config", "user.name", "Docnav Test"]);
      writeFileSync(join(tempDir, "tracked.mjs"), "console.log('one');\n", "utf8");
      runGit(tempDir, ["add", "tracked.mjs"]);
      runGit(tempDir, ["commit", "-m", "initial"]);

      const result = locateBaselineCommit({
        cwd: tempDir,
        scanInputPaths: ["*.mjs"]
      });

      assert.equal(result.ok, false);
      assert.match(result.error, /no-baseline-commit/);
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it("detectTextOnlyChange includes uncommitted and untracked scan inputs", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-scope-"));
    try {
      runGit(tempDir, ["init"]);
      runGit(tempDir, ["config", "user.email", "docnav@example.test"]);
      runGit(tempDir, ["config", "user.name", "Docnav Test"]);
      writeFileSync(join(tempDir, "tracked.mjs"), "console.log('one');\n", "utf8");
      runGit(tempDir, ["add", "tracked.mjs"]);
      runGit(tempDir, ["commit", "-m", "initial"]);
      const baselineSha = runGit(tempDir, ["rev-parse", "HEAD"]).stdout.trim();

      writeFileSync(join(tempDir, "tracked.mjs"), "console.log('two');\n", "utf8");
      writeFileSync(join(tempDir, "new.mjs"), "console.log('new');\n", "utf8");

      const scope = detectTextOnlyChange({
        baselineSha,
        cwd: tempDir,
        scanInputPaths: ["*.mjs"]
      });

      assert.equal(scope.changed, true);
      assert.ok(scope.changedFiles.includes("tracked.mjs"));
      assert.ok(scope.changedFiles.includes("new.mjs"));
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Code area CPD minimum tokens 测试
// ═══════════════════════════════════════════════════════════════════════
