import { mkdirSync } from "node:fs";
import { join } from "node:path";

import { DEFAULT_CONFIG } from "./config.ts";
import { locateBaselineCommit } from "./baseline.ts";
import { getChangedFileList, type ChangedFilesOptions } from "./files.ts";
import { validateMetrics } from "./schema.ts";
import { booleanOption, parseScriptArgs, stringOption, type ScriptArgToken } from "../args.ts";
import { gitCommitDate as readGitCommitDate, gitCommitTitle as readGitCommitTitle, gitHeadSha } from "../git.ts";
import { writeTextFile } from "../fs.ts";
import { toNdjson } from "../ndjson.ts";
import { parsePositiveInteger } from "../types.ts";
import { writeQualityJsonArtifact } from "./artifacts.ts";
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
  const parsed = parseScriptArgs({
    args: argv,
    options: {
      baseline: { type: "string" },
      "changed-files": { type: "string" },
      "top-n": { type: "string" },
      "artifact-dir": { type: "string" },
      "skip-baseline": { type: "boolean" },
      "with-baseline": { type: "boolean" },
      help: { type: "boolean" }
    }
  });

  if (booleanOption(parsed.values, "help")) {
    printHelp();
    process.exit(0);
  }

  const baseline = stringOption(parsed.values, "baseline") ?? null;
  return {
    artifactDir: stringOption(parsed.values, "artifact-dir") ?? DEFAULT_CONFIG.artifactDir,
    baseline,
    changedFiles: stringOption(parsed.values, "changed-files") ?? null,
    skipBaseline: resolveSkipBaseline(parsed.tokens, baseline === null),
    topN: parsePositiveInteger(stringOption(parsed.values, "top-n") ?? String(DEFAULT_CONFIG.report.topN), "--top-n")
  };
}

export function prepareArtifactDirs(artifactDir: string): { rawDir: string } {
  const rawDir = join(artifactDir, "raw");
  mkdirSync(artifactDir, { recursive: true });
  mkdirSync(rawDir, { recursive: true });
  return { rawDir };
}

export async function initializeToolResults(rootDir: string): Promise<ToolAvailability[]> {
  console.log("Checking tool availability...");
  const toolResults = await checkTools(rootDir);
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
    console.log("Skipping baseline scan (default; use --with-baseline or --baseline <sha> to compare).");
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

export function resolveChangedFilesForScan({
  opts,
  root,
  scope,
  collectChangedFiles = getChangedFileList
}: {
  collectChangedFiles?: (opts: ChangedFilesOptions, rootDir: string) => string[];
  opts: ChangedFilesOptions;
  root: string;
  scope: ChangeScope;
}): string[] {
  if (opts.changedFiles) {
    return collectChangedFiles(opts, root);
  }

  if (scope.changedFiles.length > 0 || !scope.changed) {
    return scope.changedFiles;
  }

  return collectChangedFiles(opts, root);
}

export function writeBaselineRawOutputs(rawDir: string, baselineSnapshot: BaselineSnapshot): void {
  const baselineRawDir = join(rawDir, "baseline");
  mkdirSync(baselineRawDir, { recursive: true });
  writeQualityJsonArtifact(join(baselineRawDir, "baseline-fingerprints.json"), baselineSnapshot.fingerprints);

  if (baselineSnapshot.fileMetrics) {
    writeQualityJsonArtifact(join(baselineRawDir, "baseline-scc-files.json"), baselineSnapshot.fileMetrics);
  }
  if (baselineSnapshot.functionMetrics) {
    writeQualityJsonArtifact(join(baselineRawDir, "baseline-lizard-functions.json"), baselineSnapshot.functionMetrics);
  }
  if (baselineSnapshot.duplicateCode) {
    writeQualityJsonArtifact(join(baselineRawDir, "baseline-cpd-fragments.json"), baselineSnapshot.duplicateCode);
  }
  if (baselineSnapshot.aggregates) {
    writeQualityJsonArtifact(join(baselineRawDir, "baseline-aggregates.json"), baselineSnapshot.aggregates);
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
  writeQualityJsonArtifact(metricsPath, metrics);
  console.log(`  metrics.json → ${metricsPath}`);

  const reportPath = join(artifactDir, "report.md");
  writeTextFile(
    reportPath,
    generateMarkdownReport(metrics, topN, { timeZone: DEFAULT_CONFIG.report.timeZone })
  );
  console.log(`  report.md → ${reportPath}`);

  const warningsPath = join(artifactDir, "warnings.ndjson");
  writeTextFile(warningsPath, toNdjson(metrics.warnings.changed));
  console.log(`  warnings.ndjson → ${warningsPath}`);

  const allWarningsPath = join(artifactDir, "warnings-all.ndjson");
  writeTextFile(allWarningsPath, toNdjson(metrics.warnings.all));
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
  return gitHeadSha(cwd) ?? "unknown";
}

export function getGitCommitTitle(sha: string, cwd: string): string | null {
  return readGitCommitTitle(sha, cwd);
}

function printHelp() {
  console.log(`
Docnav Code Quality Observability — 非阻断代码质量观测

Usage: node scripts/quality-scan.ts [options]

Options:
  --baseline <sha>        Generate baseline delta from an explicit commit SHA (opt-in)
  --with-baseline         Auto-detect and scan previous-code baseline (slower, opt-in)
  --changed-files <file>  File containing list of changed files (one per line)
  --top-n <n>             Top N for rankings (default: ${DEFAULT_CONFIG.report.topN})
  --artifact-dir <dir>    Artifact output directory (default: ${DEFAULT_CONFIG.artifactDir})
  --skip-baseline         Skip baseline commit detection and scan (default)
  --help                  Show this help

Output:
  metrics.json            Machine-readable quality metrics
  report.md               Human-readable Markdown summary
  warnings.ndjson         Changed warning records when baseline comparison is enabled (newline-delimited JSON)
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
  return readGitCommitDate(sha, cwd);
}

function resolveSkipBaseline(tokens: readonly ScriptArgToken[], defaultValue: boolean): boolean {
  let skipBaseline = defaultValue;
  for (const token of tokens) {
    if (token.kind !== "option") continue;
    if (token.name === "skip-baseline") {
      skipBaseline = true;
    } else if (token.name === "baseline" || token.name === "with-baseline") {
      skipBaseline = false;
    }
  }
  return skipBaseline;
}

function formatCommitLabel(sha: string, title: string | null): string {
  return title ? `${sha} - ${title}` : sha;
}
