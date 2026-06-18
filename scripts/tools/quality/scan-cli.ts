import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { DEFAULT_CONFIG } from "./config.ts";
import { locateBaselineCommit } from "./baseline.ts";
import { validateMetrics } from "./schema.ts";
import type {
  BaselineSnapshot,
  CodeAreaFingerprint,
  FatalIssue,
  QualityMetrics,
  ToolAvailability,
  ToolInfo
} from "./schema.ts";
import { generateMarkdownReport } from "./report/index.ts";
import { checkTools } from "./tools/index.ts";

export type QualityScanOptions = {
  artifactDir: string;
  baseline: string | null;
  changedFiles: string | null;
  skipBaseline: boolean;
  topN: number;
};

type ChangeScope = {
  changed: boolean;
  changedFiles: string[];
};

export function parseArgs(argv = process.argv.slice(2)): QualityScanOptions {
  const opts: QualityScanOptions = {
    baseline: null,
    changedFiles: null,
    topN: DEFAULT_CONFIG.report.topN,
    artifactDir: DEFAULT_CONFIG.artifactDir,
    skipBaseline: false
  };

  for (let i = 0; i < argv.length; i++) {
    switch (argv[i]) {
      case "--baseline":
        opts.baseline = argv[++i];
        break;
      case "--changed-files":
        opts.changedFiles = argv[++i];
        break;
      case "--top-n":
        opts.topN = parseInt(argv[++i], 10);
        break;
      case "--artifact-dir":
        opts.artifactDir = argv[++i];
        break;
      case "--skip-baseline":
        opts.skipBaseline = true;
        break;
      case "--help":
        printHelp();
        process.exit(0);
    }
  }

  return opts;
}

export function prepareArtifactDirs(artifactDir: string): { rawDir: string } {
  const rawDir = join(artifactDir, "raw");
  mkdirSync(artifactDir, { recursive: true });
  mkdirSync(rawDir, { recursive: true });
  return { rawDir };
}

export function initializeToolResults(rootDir: string): ToolAvailability[] {
  console.log("Checking tool availability...");
  const toolResults = checkTools(rootDir);
  const availableTools = toolResults.filter((tool) => tool.available);
  console.log(`  Available: ${availableTools.map((tool) => tool.name).join(", ") || "none"}`);

  for (const tool of toolResults) {
    if (tool.available) continue;

    console.log(`  ⚠️  ${tool.name} validation failed: ${tool.error || "not found"} (skipped)`);
  }

  return toolResults;
}

export function collectToolMetadata(toolResults: ToolAvailability[]): ToolInfo[] {
  return toolResults
    .filter((tool): tool is ToolAvailability & { version: string } => tool.available && typeof tool.version === "string")
    .map((tool) => ({
      name: tool.name,
      version: tool.version,
      source: tool.source
    }));
}

export function configureBaseline({
  metrics,
  opts,
  tools,
  root
}: {
  metrics: QualityMetrics;
  opts: QualityScanOptions;
  root: string;
  tools: ToolInfo[];
}): void {
  if (opts.baseline) {
    metrics.baseline = createGeneratedBaseline(opts.baseline, "explicit", tools, root);
    return;
  }

  if (opts.skipBaseline) {
    metrics.baseline = {
      status: "baseline-skipped",
      commitSha: null,
      commitDate: null,
      metadata: null
    };
    return;
  }

  console.log("Locating baseline commit...");
  const baselineResult = locateBaselineCommit({
    cwd: root,
    scanInputPaths: DEFAULT_CONFIG.include
  });

  if (baselineResult.ok) {
    const baselineTitle = getGitCommitTitle(baselineResult.sha, root);
    console.log(`  Baseline commit: ${formatCommitLabel(baselineResult.sha, baselineTitle)} (${baselineResult.reason})`);
    metrics.baseline = createGeneratedBaseline(
      baselineResult.sha,
      baselineResult.reason,
      tools,
      root,
      baselineResult.date ?? null,
      baselineTitle
    );
  } else {
    console.log(`  ⚠️  No baseline: ${baselineResult.error}`);
    const baselineStatus = baselineResult.error?.includes("no-baseline-commit")
      ? "no-baseline-commit"
      : "history-unavailable";
    metrics.baseline = {
      status: baselineStatus,
      commitSha: null,
      commitDate: null,
      metadata: null
    };
  }
}

export function setComparisonStatus(metrics: QualityMetrics, scope: ChangeScope): void {
  if (metrics.baseline.status === "generated" && metrics.baseline.commitSha) {
    if (!scope.changed) {
      metrics.comparisonStatus = "input-unchanged";
      console.log("  Comparison: input-unchanged (text-only or non-scan-input change)");
    } else {
      metrics.comparisonStatus = "compared";
      console.log(`  Comparison: ${scope.changedFiles.length} files changed in scan scope`);
    }
  } else {
    metrics.comparisonStatus = "baseline-unavailable";
    console.log("  Comparison: baseline-unavailable");
  }
}

export function writeBaselineRawOutputs(rawDir: string, baselineSnapshot: BaselineSnapshot): void {
  const baselineRawDir = join(rawDir, "baseline");
  mkdirSync(baselineRawDir, { recursive: true });
  writeJson(join(baselineRawDir, "baseline-fingerprints.json"), baselineSnapshot.fingerprints);

  if (baselineSnapshot.fileMetrics) {
    writeJson(join(baselineRawDir, "baseline-scc-files.json"), baselineSnapshot.fileMetrics);
  }
  if (baselineSnapshot.functionMetrics) {
    writeJson(join(baselineRawDir, "baseline-lizard-functions.json"), baselineSnapshot.functionMetrics);
  }
  if (baselineSnapshot.duplicateCode) {
    writeJson(join(baselineRawDir, "baseline-cpd-fragments.json"), baselineSnapshot.duplicateCode);
  }
  if (baselineSnapshot.aggregates) {
    writeJson(join(baselineRawDir, "baseline-aggregates.json"), baselineSnapshot.aggregates);
  }
}

export function writeArtifacts({
  artifactDir,
  metrics,
  topN
}: {
  artifactDir: string;
  metrics: QualityMetrics;
  topN: number;
}): void {
  console.log("Writing artifacts...");

  const metricsPath = join(artifactDir, "metrics.json");
  writeJson(metricsPath, metrics);
  console.log(`  metrics.json → ${metricsPath}`);

  const reportPath = join(artifactDir, "report.md");
  writeFileSync(
    reportPath,
    generateMarkdownReport(metrics, topN, { timeZone: DEFAULT_CONFIG.report.timeZone }),
    "utf8"
  );
  console.log(`  report.md → ${reportPath}`);

  const warningsPath = join(artifactDir, "warnings.ndjson");
  writeFileSync(warningsPath, toNdjson(metrics.warnings.changed), "utf8");
  console.log(`  warnings.ndjson → ${warningsPath}`);

  const allWarningsPath = join(artifactDir, "warnings-all.ndjson");
  writeFileSync(allWarningsPath, toNdjson(metrics.warnings.all), "utf8");
  console.log(`  warnings-all.ndjson → ${allWarningsPath}`);
}

export function printSummary(metrics: QualityMetrics): void {
  console.log("");
  console.log("─".repeat(60));
  console.log("Summary:");
  console.log(`  Files: ${metrics.fileMetrics.length}`);
  console.log(`  Functions: ${metrics.functionMetrics.length}`);
  console.log(`  Duplicate fragments: ${metrics.duplicateCode.length}`);
  console.log(`  Warnings: ${metrics.warnings.all.length} all`);
  console.log(`  Changed warnings: ${metrics.warnings.changed.length}`);
  console.log(`  Regression warnings: ${metrics.warnings.regressions.length}`);
  console.log(`  Baseline status: ${metrics.baseline.status}`);
  console.log(`  Comparison status: ${metrics.comparisonStatus}`);
  console.log("─".repeat(60));
}

export function validateOutput(metrics: QualityMetrics) {
  const validation = validateMetrics(metrics);
  if (validation.valid) return validation;

  console.log("");
  console.log("❌ Metrics validation errors:");
  for (const err of validation.errors) {
    console.log(`  - ${err}`);
  }
  return validation;
}

export function logFingerprints(fingerprints: Record<string, CodeAreaFingerprint>): void {
  console.log("  Input fingerprints:");
  for (const [area, fingerprint] of Object.entries(fingerprints)) {
    console.log(`    ${area}: ${fingerprint.fileCount} files, ${fingerprint.fingerprint}`);
  }
}

export function formatFatalIssue(issue: FatalIssue): string {
  return `${issue.tool} ${issue.phase}: ${issue.error}`;
}

export function getGitSha(cwd: string): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd, encoding: "utf8", windowsHide: true });
  return (result.stdout || "").trim() || "unknown";
}

export function getGitCommitTitle(sha: string, cwd: string): string | null {
  const result = spawnSync("git", ["log", "--format=%s", "--max-count=1", sha], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  return (result.stdout || "").trim() || null;
}

function printHelp() {
  console.log(`
Docnav Code Quality Observability — 非阻断代码质量观测

Usage: node scripts/quality-scan.ts [options]

Options:
  --baseline <sha>        Baseline commit SHA (default: auto-detect from git history)
  --changed-files <file>  File containing list of changed files (one per line)
  --top-n <n>             Top N for rankings (default: ${DEFAULT_CONFIG.report.topN})
  --artifact-dir <dir>    Artifact output directory (default: ${DEFAULT_CONFIG.artifactDir})
  --skip-baseline         Skip baseline commit detection and scan
  --help                  Show this help

Output:
  metrics.json            Machine-readable quality metrics
  report.md               Human-readable Markdown summary
  warnings.ndjson         Changed warning records for CI annotations (newline-delimited JSON)
  warnings-all.ndjson     Full warning records for local/governance use
  raw/                    Raw tool outputs (Lizard, scc, PMD CPD)

⚠️  Non-blocking: Lizard/scc/PMD CPD metric values do not cause command failure.
   Clippy remains the Rust blocking lint gate.
`);
}

function createGeneratedBaseline(
  commitSha: string,
  selectionReason: string,
  tools: ToolInfo[],
  root: string,
  commitDate: string | null = null,
  commitTitle: string | null = null
): QualityMetrics["baseline"] {
  const resolvedDate = commitDate || getGitCommitDate(commitSha, root);
  const resolvedTitle = commitTitle || getGitCommitTitle(commitSha, root);
  return {
    status: "generated",
    commitSha,
    commitDate: resolvedDate,
    metadata: {
      commitSha,
      commitDate: resolvedDate || "unknown",
      commitTitle: resolvedTitle,
      selectionReason,
      configVersion: DEFAULT_CONFIG.version,
      toolMetadata: tools
    }
  };
}

function getGitCommitDate(sha: string, cwd: string): string | null {
  const result = spawnSync("git", ["log", "--format=%aI", "--max-count=1", sha], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  return (result.stdout || "").trim() || null;
}

function writeJson(filePath: string, value: unknown): void {
  writeFileSync(filePath, JSON.stringify(value, null, 2), "utf8");
}

function toNdjson(values: unknown[]): string {
  return values.length === 0 ? "" : `${values.map((value) => JSON.stringify(value)).join("\n")}\n`;
}

function formatCommitLabel(sha: string, title: string | null): string {
  return title ? `${sha} - ${title}` : sha;
}
