import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import {
  METRICS_SCHEMA_VERSION,
  createEmptyMetrics,
  validateMetrics,
  BASELINE_STATUSES,
  COMPARISON_STATUSES,
  WARNING_LEVELS
} from "../../scripts/quality/schema.mjs";

describe("config", () => {
  it("DEFAULT_CONFIG has version field", () => {
    assert.ok(DEFAULT_CONFIG.version);
    assert.ok(typeof DEFAULT_CONFIG.version === "string");
  });

  it("DEFAULT_CONFIG defines all 6 default code areas", () => {
    const expected = [
      "rust-production",
      "rust-tests",
      "node-production-scripts",
      "node-validation-smoke",
      "fixtures-examples",
      "generated"
    ];
    for (const name of expected) {
      assert.ok(DEFAULT_CONFIG.codeAreas[name], `code area "${name}" is missing`);
      const def = DEFAULT_CONFIG.codeAreas[name];
      assert.ok(def.description, `"${name}" missing description`);
      assert.ok(Array.isArray(def.globs), `"${name}" globs must be array`);
      assert.ok(typeof def.warningPolicy === "string", `"${name}" missing warningPolicy`);
    }
  });

  it("DEFAULT_CONFIG excludeDirs covers required exclusions", () => {
    const requiredExclusions = [".git", "target", "node_modules", ".venv", "dist", "build"];
    for (const d of requiredExclusions) {
      assert.ok(DEFAULT_CONFIG.excludeDirs.includes(d), `"${d}" not in excludeDirs`);
    }
  });

  it("DEFAULT_CONFIG uses the quality artifact directory", () => {
    assert.equal(DEFAULT_CONFIG.artifactDir, "artifacts/docnav-quality");
  });

  it("DEFAULT_CONFIG pmdCpd has per-code-area minimumTokens", () => {
    const tokens = DEFAULT_CONFIG.pmdCpd.minimumTokens;
    assert.ok(typeof tokens["rust-production"] === "number");
    assert.ok(typeof tokens["rust-tests"] === "number");
    assert.ok(typeof tokens["node-production-scripts"] === "number");
    assert.ok(typeof tokens["node-validation-smoke"] === "number");
    assert.ok(typeof tokens["fixtures-examples"] === "number");
    assert.ok(typeof tokens["generated"] === "number");
    // production 应该更敏感（更小的 minimum tokens）
    assert.ok(tokens["rust-production"] <= tokens["rust-tests"],
      "production CPD should be at least as sensitive as tests");
  });

  it("DEFAULT_CONFIG warning policies are valid", () => {
    const validPolicies = ["strict", "moderate", "relaxed", "watchlist-only", "exclude-warnings"];
    for (const [name, def] of Object.entries(DEFAULT_CONFIG.codeAreas)) {
      assert.ok(validPolicies.includes(def.warningPolicy),
        `"${name}" policy "${def.warningPolicy}" is not valid`);
    }
  });

  it("DEFAULT_CONFIG warning policies follow expected strictness", () => {
    // production areas should use strict
    assert.equal(DEFAULT_CONFIG.codeAreas["rust-production"].warningPolicy, "strict");
    // test areas should be relaxed
    assert.equal(DEFAULT_CONFIG.codeAreas["rust-tests"].warningPolicy, "relaxed");
    // generated should exclude warnings
    assert.equal(DEFAULT_CONFIG.codeAreas["generated"].warningPolicy, "exclude-warnings");
    // fixtures should be watchlist-only
    assert.equal(DEFAULT_CONFIG.codeAreas["fixtures-examples"].warningPolicy, "watchlist-only");
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Metrics schema 测试
// ═══════════════════════════════════════════════════════════════════════

describe("metrics schema", () => {
  it("METRICS_SCHEMA_VERSION is set", () => {
    assert.ok(METRICS_SCHEMA_VERSION);
    assert.ok(/^\d+\.\d+\.\d+$/.test(METRICS_SCHEMA_VERSION));
  });

  it("BASELINE_STATUSES contains all required statuses", () => {
    const expected = [
      "generated",
      "history-unavailable",
      "no-baseline-commit",
      "baseline-materialization-failed",
      "baseline-scan-failed",
      "baseline-skipped"
    ];
    for (const s of expected) {
      assert.ok(BASELINE_STATUSES.includes(s), `"${s}" missing from BASELINE_STATUSES`);
    }
  });

  it("COMPARISON_STATUSES contains all required statuses", () => {
    const expected = ["compared", "input-unchanged", "baseline-unavailable"];
    for (const s of expected) {
      assert.ok(COMPARISON_STATUSES.includes(s), `"${s}" missing from COMPARISON_STATUSES`);
    }
  });

  it("WARNING_LEVELS contains expected levels", () => {
    assert.ok(WARNING_LEVELS.includes("info"));
    assert.ok(WARNING_LEVELS.includes("warning"));
    assert.ok(WARNING_LEVELS.includes("error"));
  });

  it("createEmptyMetrics returns valid skeleton", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc123",
      configVersion: "0.1.0",
      tools: [{ name: "lizard", version: "1.0", source: "uv" }],
      scope: { include: ["*.rs"], excludeDirs: ["target"], generatedFiles: [] }
    });

    assert.ok(metrics.metadata);
    assert.equal(metrics.metadata.schemaVersion, METRICS_SCHEMA_VERSION);
    assert.ok(metrics.metadata.timestamp);
    assert.ok(Array.isArray(metrics.fileMetrics));
    assert.ok(Array.isArray(metrics.functionMetrics));
    assert.ok(Array.isArray(metrics.duplicateCode));
    assert.ok(Array.isArray(metrics.warnings));
    assert.ok(metrics.aggregates);
  });

  it("validateMetrics accepts valid skeleton", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc123",
      configVersion: "0.1.0",
      tools: [{ name: "lizard", version: "1.0", source: "uv" }],
      scope: { include: ["*.rs"], excludeDirs: ["target"], generatedFiles: [] }
    });

    const result = validateMetrics(metrics);
    assert.ok(result.valid, `expected valid but got: ${result.errors.join(", ")}`);
  });

  it("validateMetrics rejects missing metadata", () => {
    const result = validateMetrics({});
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("metadata")));
  });

  it("validateMetrics rejects invalid baseline status", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.baseline.status = "invalid-status";

    const result = validateMetrics(metrics);
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("baseline.status")));
  });

  it("validateMetrics rejects invalid comparison status", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.comparisonStatus = "invalid";

    const result = validateMetrics(metrics);
    assert.ok(!result.valid);
    assert.ok(result.errors.some((e) => e.includes("comparisonStatus")));
  });

  it("validateMetrics validates warning records", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.warnings = [
      { level: "INVALID", ruleId: null, message: null }
    ];

    const result = validateMetrics(metrics);
    assert.ok(!result.valid);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// 文件分类测试
// ═══════════════════════════════════════════════════════════════════════
