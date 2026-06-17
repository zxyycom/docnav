import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { DEFAULT_CONFIG } from "./config.mjs";
import { locateBaselineCommit } from "./baseline.mjs";
import { validateMetrics } from "./schema.mjs";
import { generateMarkdownReport } from "./report/index.mjs";
import { checkTools } from "./tools/index.mjs";

export function parseArgs(argv = process.argv.slice(2)) {
  const opts = {
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

export function prepareArtifactDirs(artifactDir) {
  const rawDir = join(artifactDir, "raw");
  mkdirSync(artifactDir, { recursive: true });
  mkdirSync(rawDir, { recursive: true });
  return { rawDir };
}

export function initializeToolResults(rootDir) {
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

export function collectToolMetadata(toolResults) {
  return toolResults
    .filter((tool) => tool.available && tool.version)
    .map((tool) => ({
      name: tool.name,
      version: tool.version,
      source: tool.source
    }));
}

export function configureBaseline({ metrics, opts, tools, fatalIssues, root }) {
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
    console.log(`  Baseline commit: ${baselineResult.sha} (${baselineResult.reason})`);
    metrics.baseline = createGeneratedBaseline(
      baselineResult.sha,
      baselineResult.reason,
      tools,
      root,
      baselineResult.date
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

export function setComparisonStatus(metrics, scope) {
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

export function writeBaselineRawOutputs(rawDir, baselineSnapshot) {
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

export function writeArtifacts({ artifactDir, metrics, topN }) {
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
  const ndjson = metrics.warnings.map((warning) => JSON.stringify(warning)).join("\n") + "\n";
  writeFileSync(warningsPath, ndjson, "utf8");
  console.log(`  warnings.ndjson → ${warningsPath}`);
}

export function printSummary(metrics) {
  console.log("");
  console.log("─".repeat(60));
  console.log("Summary:");
  console.log(`  Files: ${metrics.fileMetrics.length}`);
  console.log(`  Functions: ${metrics.functionMetrics.length}`);
  console.log(`  Duplicate fragments: ${metrics.duplicateCode.length}`);
  console.log(`  Warnings: ${metrics.warnings.length}`);
  console.log(`  Baseline status: ${metrics.baseline.status}`);
  console.log(`  Comparison status: ${metrics.comparisonStatus}`);
  if (metrics.trends.length > 0) {
    console.log(`  Trends: ${metrics.trends.length} deltas`);
  }
  console.log("─".repeat(60));
}

export function validateOutput(metrics) {
  const validation = validateMetrics(metrics);
  if (validation.valid) return validation;

  console.log("");
  console.log("❌ Metrics validation errors:");
  for (const err of validation.errors) {
    console.log(`  - ${err}`);
  }
  return validation;
}

export function logFingerprints(fingerprints) {
  console.log("  Input fingerprints:");
  for (const [area, fingerprint] of Object.entries(fingerprints)) {
    console.log(`    ${area}: ${fingerprint.fileCount} files, ${fingerprint.fingerprint}`);
  }
}

export function formatFatalIssue(issue) {
  return `${issue.tool} ${issue.phase}: ${issue.error}`;
}

export function getGitSha(cwd) {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd, encoding: "utf8", windowsHide: true });
  return (result.stdout || "").trim() || "unknown";
}

function printHelp() {
  console.log(`
Docnav Code Quality Observability — 非阻断代码质量观测

Usage: node scripts/quality-scan.mjs [options]

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
  warnings.ndjson         Warning records (newline-delimited JSON)
  raw/                    Raw tool outputs (Lizard, scc, PMD CPD)

⚠️  Non-blocking: Lizard/scc/PMD CPD metric values do not cause command failure.
   Clippy remains the Rust blocking lint gate.
`);
}

function createGeneratedBaseline(commitSha, selectionReason, tools, root, commitDate = null) {
  const resolvedDate = commitDate || getGitCommitDate(commitSha, root);
  return {
    status: "generated",
    commitSha,
    commitDate: resolvedDate,
    metadata: {
      commitSha,
      commitDate: resolvedDate || "unknown",
      selectionReason,
      configVersion: DEFAULT_CONFIG.version,
      toolMetadata: tools
    }
  };
}

function getGitCommitDate(sha, cwd) {
  const result = spawnSync("git", ["log", "--format=%aI", "--max-count=1", sha], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  return (result.stdout || "").trim() || null;
}

function writeJson(filePath, value) {
  writeFileSync(filePath, JSON.stringify(value, null, 2), "utf8");
}
