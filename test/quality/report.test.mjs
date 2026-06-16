import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { createEmptyMetrics } from "../../scripts/quality/schema.mjs";
import { generateMarkdownReport } from "../../scripts/quality/report/index.mjs";
import { formatReportTimestamp } from "../../scripts/quality/report/summary.mjs";

describe("report generation", () => {
  it("generates markdown with required sections", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc123",
      configVersion: "0.1.0",
      tools: [{ name: "lizard", version: "1.23.0", source: "uv" }],
      scope: { include: ["*.rs"], excludeDirs: ["target"], generatedFiles: [] }
    });

    // Add some file metrics
    metrics.fileMetrics = [
      { path: "crates/docnav/src/lib.rs", language: "Rust", codeArea: "rust-production", lines: 100, codeLines: 80, complexity: { value: 15, source: "scc" }, isChanged: false },
      { path: "scripts/cargo.mjs", language: "JavaScript", codeArea: "node-production-scripts", lines: 50, codeLines: 40, complexity: { value: 5, source: "scc" }, isChanged: true }
    ];

    metrics.aggregates.overall = { totalFiles: 2, totalLines: 150, totalCodeLines: 120, totalFunctions: 0 };
    metrics.aggregates.byLanguage = [
      { language: "Rust", files: 1, lines: 100, codeLines: 80, commentLines: 10, blankLines: 10, complexitySource: "scc" }
    ];

    const report = generateMarkdownReport(metrics, 10);

    assert.ok(report.includes("Docnav Code Quality Snapshot"));
    assert.ok(report.includes("非阻断观测快照"));
    assert.ok(report.includes("扫描信息"));
    assert.ok(report.includes("仓库体量"));
    assert.ok(report.includes("abc123"));

    // Should not include trend data since comparison is baseline-unavailable
    assert.ok(report.includes("Baseline 不可用") || report.includes("baseline 不可用"));
  });

  it("markdown report handles empty metrics gracefully", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    const report = generateMarkdownReport(metrics, 10);
    assert.ok(report.length > 0);
    assert.ok(report.includes("Docnav Code Quality Snapshot"));
  });

  it("markdown report includes input-unchanged note", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.comparisonStatus = "input-unchanged";

    const report = generateMarkdownReport(metrics, 10);
    assert.ok(report.includes("代码输入未变化"));
  });

  it("markdown report explains when baseline scan was skipped", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.baseline.status = "baseline-skipped";
    metrics.comparisonStatus = "baseline-unavailable";

    const report = generateMarkdownReport(metrics, 10);

    assert.match(report, /Baseline scan was skipped \(`baseline-skipped`\)/);
  });

  it("markdown report footer notes non-blocking nature", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });

    const report = generateMarkdownReport(metrics, 10);
    assert.ok(report.includes("非阻断观测"));
    assert.ok(report.includes("Clippy"));
  });

  it("markdown report renders timestamps in the configured report time zone", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.metadata.timestamp = "2026-06-16T01:02:03.000Z";

    const report = generateMarkdownReport(metrics, 10, { timeZone: "Asia/Shanghai" });

    assert.ok(report.includes("2026-06-16 09:02:03 GMT+08:00 (Asia/Shanghai; source 2026-06-16T01:02:03.000Z)"));
  });

  it("markdown report rejects invalid report time zone configuration", () => {
    assert.throws(
      () => formatReportTimestamp("2026-06-16T01:02:03.000Z", "Not/AZone"),
      /Invalid time zone specified/
    );
  });

  it("markdown report lists duplicate fragment locations with line ranges", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.duplicateCode = [
      {
        id: 1,
        tokenCount: 135,
        lineCount: 29,
        locations: [
          {
            path: "crates/docnav/src/a.rs",
            startLine: 10,
            endLine: 38,
            codeArea: "rust-production"
          },
          {
            path: "crates/docnav/src/b.rs",
            startLine: 42,
            endLine: 70,
            codeArea: "rust-production"
          }
        ],
        codeAreas: ["rust-production"],
        hitsChangedScope: true
      }
    ];

    const report = generateMarkdownReport(metrics, 10);

    assert.match(report, /\*\*Fragment #1\*\*: 135 tokens, 29 lines/);
    assert.match(report, /Locations \(2\):/);
    assert.match(report, /crates\/docnav\/src\/a\.rs:10-38 \(rust-production\)/);
    assert.match(report, /crates\/docnav\/src\/b\.rs:42-70 \(rust-production\)/);
    assert.match(report, /命中 changed scope/);
  });

  it("markdown report rejects duplicate fragments without location code areas", () => {
    const metrics = createEmptyMetrics({
      repository: "/test/repo",
      commitSha: "abc",
      configVersion: "0.1.0",
      tools: [],
      scope: { include: [], excludeDirs: [], generatedFiles: [] }
    });
    metrics.duplicateCode = [
      {
        id: 1,
        tokenCount: 135,
        lineCount: 29,
        locations: [
          {
            path: "crates/docnav/src/a.rs",
            startLine: 10,
            endLine: 38,
            codeArea: "unknown"
          }
        ],
        codeAreas: ["rust-production"],
        hitsChangedScope: false
      }
    ];

    assert.throws(
      () => generateMarkdownReport(metrics, 10),
      /location is missing code area/
    );
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Trend 和 CI annotation 测试
// ═══════════════════════════════════════════════════════════════════════
