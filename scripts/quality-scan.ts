#!/usr/bin/env node

/**
 * Docnav 代码质量观测命令入口。
 *
 * 非阻断代码质量扫描：Clippy 保持 Rust 阻断 gate，Lizard/scc/PMD CPD
 * 生成非阻断代码质量快照、warning 和报告。
 *
 * 来源：openspec/changes/implement-code-quality-observability/
 */

import { rmSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath, pathToFileURL } from "node:url";
import { randomUUID } from "node:crypto";

import { DEFAULT_CONFIG } from "./tools/quality/config.ts";
import { createEmptyMetrics } from "./tools/quality/schema.ts";
import type { BaselineSnapshot, FatalIssue, QualityMetrics, ToolAvailability } from "./tools/quality/schema.ts";
import { errorMessage } from "./tools/types.ts";
import { classifyFiles } from "./tools/quality/classify.ts";
import {
  materializeBaseline,
  detectTextOnlyChange
} from "./tools/quality/baseline.ts";
import { generateWarnings } from "./tools/quality/warnings.ts";
import {
  collectScanFiles,
  getChangedFileList,
  buildFingerprints
} from "./tools/quality/files.ts";
import { scanCurrentRevision } from "./tools/quality/scan-runner.ts";
import { scanBaselineRevision } from "./tools/quality/baseline-scan.ts";
import { generateTrends } from "./tools/quality/trends.ts";
import {
  parseArgs,
  prepareArtifactDirs,
  initializeToolResults,
  collectToolMetadata,
  configureBaseline,
  setComparisonStatus,
  writeBaselineRawOutputs,
  writeArtifacts,
  printSummary,
  validateOutput,
  logFingerprints,
  formatFatalIssue,
  getGitSha
} from "./tools/quality/scan-cli.ts";

export { scanBaselineRevision } from "./tools/quality/baseline-scan.ts";
export { buildAggregates } from "./tools/quality/aggregate.ts";
export { generateTrends } from "./tools/quality/trends.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

async function main() {
  const opts = parseArgs();

  console.log("Docnav Code Quality Observability");
  console.log("Non-blocking snapshot — metric values do not cause failure.");
  console.log("");

  const artifactDir = resolve(root, opts.artifactDir);
  const { rawDir } = prepareArtifactDirs(artifactDir);

  const commitSha = getGitSha(root);
  const toolResults = initializeToolResults(root);
  const tools = collectToolMetadata(toolResults);

  console.log("Collecting scan inputs...");
  const scanFiles = collectScanFiles(root, DEFAULT_CONFIG);
  console.log(`  Found ${scanFiles.length} files in scan scope`);

  const fileMap = classifyFiles(scanFiles, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles);
  const areaNames = Array.from(fileMap.keys());
  console.log(`  Code areas: ${areaNames.join(", ")}`);

  const fingerprints = buildFingerprints(fileMap, root);
  logFingerprints(fingerprints);

  const metrics = createEmptyMetrics({
    repository: root,
    commitSha,
    configVersion: DEFAULT_CONFIG.version,
    tools,
    scope: {
      include: DEFAULT_CONFIG.include,
      excludeDirs: DEFAULT_CONFIG.excludeDirs,
      generatedFiles: DEFAULT_CONFIG.generatedFiles
    }
  });
  metrics.currentFingerprints = fingerprints;

  const fatalIssues: FatalIssue[] = [];
  configureBaseline({ metrics, opts, tools, root });

  const scope = detectTextOnlyChange({
    baselineSha: metrics.baseline.commitSha,
    cwd: root,
    scanInputPaths: DEFAULT_CONFIG.include
  });
  const changedFiles = opts.changedFiles ? getChangedFileList(opts, root) : scope.changedFiles;
  console.log(`  Changed files in scan scope: ${changedFiles.length}`);

  scanCurrentRevision({
    metrics,
    toolResults,
    scanFiles,
    changedFiles,
    fileMap,
    rawDir,
    fatalIssues,
    root,
    config: DEFAULT_CONFIG
  });

  setComparisonStatus(metrics, scope);
  const baselineSnapshot = maybeScanBaseline({
    metrics,
    toolResults,
    rawDir,
    fatalIssues
  });

  console.log("Generating warnings...");
  metrics.warnings = generateWarnings({
    files: metrics.fileMetrics,
    functions: metrics.functionMetrics,
    duplicates: metrics.duplicateCode,
    config: DEFAULT_CONFIG,
    scope,
    baseline: baselineSnapshot
      ? {
          files: baselineSnapshot.fileMetrics,
          functions: baselineSnapshot.functionMetrics,
          duplicates: baselineSnapshot.duplicateCode
        }
      : null,
    comparisonStatus: metrics.comparisonStatus
  });
  console.log(`  Warnings: ${metrics.warnings.length} generated`);

  writeArtifacts({ artifactDir, metrics, topN: opts.topN });
  printSummary(metrics);
  const validation = validateOutput(metrics);
  if (!validation.valid) {
    fatalIssues.push({
      tool: "metrics",
      phase: "validation",
      error: validation.errors.join("; ")
    });
  }

  const hasFatalIssues = fatalIssues.length > 0;
  console.log("");
  console.log(hasFatalIssues ? "❌ Quality scan failed." : "✅ Quality scan complete.");
  console.log(`Artifacts in: ${artifactDir}/`);

  if (hasFatalIssues) {
    console.error("Fatal quality scan issues:");
    for (const issue of fatalIssues) {
      console.error(`  - ${formatFatalIssue(issue)}`);
    }
    process.exit(2);
  }

  process.exit(0);
}

function maybeScanBaseline({
  metrics,
  toolResults,
  rawDir,
  fatalIssues
}: {
  fatalIssues: FatalIssue[];
  metrics: QualityMetrics;
  rawDir: string;
  toolResults: ToolAvailability[];
}): BaselineSnapshot | null {
  if (fatalIssues.length > 0) {
    console.log("Skipping baseline scan because fatal current-scan errors were detected.");
    return null;
  }

  const baselineCommitSha = metrics.baseline.commitSha;
  if (metrics.baseline.status !== "generated" || !baselineCommitSha) {
    return null;
  }

  const baselineWorkDir = join(tmpdir(), `docnav-quality-baseline-${randomUUID()}`);
  console.log(`Materializing baseline ${baselineCommitSha.slice(0, 7)}...`);

  try {
    return scanMaterializedBaseline({ baselineCommitSha, metrics, toolResults, rawDir, baselineWorkDir });
  } finally {
    rmSync(baselineWorkDir, { recursive: true, force: true });
  }
}

function scanMaterializedBaseline({
  baselineCommitSha,
  metrics,
  toolResults,
  rawDir,
  baselineWorkDir
}: {
  baselineCommitSha: string;
  baselineWorkDir: string;
  metrics: QualityMetrics;
  rawDir: string;
  toolResults: ToolAvailability[];
}): BaselineSnapshot | null {
  const matResult = materializeBaseline({
    commitSha: baselineCommitSha,
    cwd: root,
    baselineWorkDir
  });

  if (!matResult.ok) {
    console.log(`  ⚠️  Baseline materialization failed: ${matResult.error}`);
    metrics.baseline.status = "baseline-materialization-failed";
    metrics.comparisonStatus = "baseline-unavailable";
    return null;
  }

  console.log(`  Baseline materialized to ${matResult.workDir}`);

  try {
    const baselineSnapshot = scanBaselineRevision(matResult.workDir, toolResults, DEFAULT_CONFIG);
    metrics.baselineFingerprints = baselineSnapshot.fingerprints;
    metrics.trends = generateTrends(metrics, baselineSnapshot);
    console.log(`  Trends: ${metrics.trends.length} trend deltas computed`);
    writeBaselineRawOutputs(rawDir, baselineSnapshot);
    return baselineSnapshot;
  } catch (err: unknown) {
    console.log(`  ⚠️  Baseline scan failed: ${errorMessage(err)}`);
    metrics.baseline.status = "baseline-scan-failed";
    metrics.comparisonStatus = "baseline-unavailable";
    return null;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((err: unknown) => {
    const message = errorMessage(err);
    console.error("Fatal error in quality scan:", message);
    if ((err instanceof Error && "code" in err && err.code === "ENOENT") || message.includes("config")) {
      process.exit(3);
    }
    process.exit(2);
  });
}
