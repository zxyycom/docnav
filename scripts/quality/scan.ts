#!/usr/bin/env node

/**
 * Docnav 代码质量观测命令入口。
 *
 * 非阻断代码质量扫描：Clippy 保持 Rust 阻断 gate，Lizard/scc/PMD CPD
 * 生成非阻断代码质量快照、warning 和报告。
 */

import { rmSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { tmpdir } from "node:os";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import { randomUUID } from "node:crypto";

import { DEFAULT_CONFIG } from "../tools/quality/config.ts";
import { createEmptyMetrics } from "../tools/quality/schema.ts";
import type { BaselineSnapshot, FatalIssue, QualityMetrics, ToolAvailability } from "../tools/quality/schema.ts";
import { errorMessage } from "../tools/types.ts";
import { classifyFiles } from "../tools/quality/classify.ts";
import {
  materializeBaseline,
  detectTextOnlyChange
} from "../tools/quality/baseline/index.ts";
import { generateWarningChannels } from "../tools/quality/warnings.ts";
import {
  collectScanFiles,
  buildFingerprints
} from "../tools/quality/files.ts";
import { scanCurrentRevision } from "../tools/quality/scan/current/index.ts";
import { scanBaselineRevision } from "../tools/quality/baseline/scan.ts";
import { generateTrends } from "../tools/quality/trends.ts";
import {
  parseArgs,
  prepareArtifactDirs,
  initializeToolResults,
  collectToolMetadata,
  configureBaseline,
  setComparisonStatus,
  resolveChangedFilesForScan,
  writeBaselineRawOutputs,
  writeArtifacts,
  printSummary,
  validateOutput,
  logFingerprints,
  formatFatalIssue,
  getGitSha,
  getGitCommitTitle
} from "../tools/quality/scan/cli/index.ts";

export { scanBaselineRevision } from "../tools/quality/baseline/scan.ts";
export { buildAggregates } from "../tools/quality/aggregate.ts";
export { generateTrends } from "../tools/quality/trends.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const timingsEnabled = process.env.DOCNAV_QUALITY_TIMINGS === "1";

async function main() {
  const timings = createTimings();
  const opts = parseArgs();

  console.log("Docnav Code Quality Observability");
  console.log("Non-blocking snapshot — metric values do not cause failure.");
  console.log("");

  const artifactDir = resolve(root, opts.artifactDir);
  const { rawDir } = timings.measure("prepare artifact dirs", () => prepareArtifactDirs(artifactDir));

  const commitSha = timings.measure("git rev-parse HEAD", () => getGitSha(root));
  const commitTitle = timings.measure("git commit title", () => getGitCommitTitle(commitSha, root));
  const toolResults = await timings.measureAsync("tool availability", () => initializeToolResults(root));
  const tools = timings.measure("tool metadata", () => collectToolMetadata(toolResults));

  console.log("Collecting scan inputs...");
  const scanFiles = timings.measure("collect scan files", () => collectScanFiles(root, DEFAULT_CONFIG));
  console.log(`  Found ${scanFiles.length} files in scan scope`);

  const fileMap = timings.measure("classify scan files", () =>
    classifyFiles(scanFiles, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles)
  );
  const areaNames = Array.from(fileMap.keys());
  console.log(`  Code areas: ${areaNames.join(", ")}`);

  const fingerprints = timings.measure("build fingerprints", () => buildFingerprints(fileMap, root));
  logFingerprints(fingerprints);

  const metrics = timings.measure("create metrics envelope", () => createEmptyMetrics({
    repository: root,
    commitSha,
    commitTitle,
    configVersion: DEFAULT_CONFIG.version,
    tools,
    scope: {
      include: DEFAULT_CONFIG.include,
      excludeDirs: DEFAULT_CONFIG.excludeDirs,
      generatedFiles: DEFAULT_CONFIG.generatedFiles
    }
  }));
  metrics.currentFingerprints = fingerprints;

  const fatalIssues: FatalIssue[] = [];
  timings.measure("configure baseline", () => configureBaseline({ metrics, opts, tools, root }));

  const scope = timings.measure("detect changed scan inputs", () => detectTextOnlyChange({
    baselineSha: metrics.baseline.commitSha,
    cwd: root,
    scanInputPaths: DEFAULT_CONFIG.include
  }));
  const changedFiles = timings.measure("resolve changed files", () =>
    resolveChangedFilesForScan({ opts, root, scope })
  );
  console.log(`  Changed files in scan scope: ${changedFiles.length}`);

  await timings.measureAsync("scan current revision", () => scanCurrentRevision({
    context: {
      metrics,
      toolResults,
      changedFiles,
      rawDir,
      fatalIssues,
      root,
      fingerprints,
      config: DEFAULT_CONFIG
    },
    scanFiles,
    fileMap
  }));

  timings.measure("set comparison status", () => setComparisonStatus(metrics, scope));
  const baselineSnapshot = await timings.measureAsync("baseline snapshot", () => maybeScanBaseline({
    metrics,
    toolResults,
    rawDir,
    fatalIssues
  }));

  console.log("Generating warnings...");
  metrics.warnings = timings.measure("generate warnings", () => generateWarningChannels({
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
  }));
  console.log(
    `  Warnings: ${metrics.warnings.all.length} all, ` +
    `${metrics.warnings.changed.length} changed, ` +
    `${metrics.warnings.regressions.length} regressions generated`
  );

  timings.measure("write artifacts", () => writeArtifacts({ artifactDir, metrics, topN: opts.topN }));
  timings.measure("print summary", () => printSummary(metrics));
  const validation = timings.measure("validate output", () => validateOutput(metrics));
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

  timings.print();
  process.exit(0);
}

async function maybeScanBaseline({
  metrics,
  toolResults,
  rawDir,
  fatalIssues
}: {
  fatalIssues: FatalIssue[];
  metrics: QualityMetrics;
  rawDir: string;
  toolResults: ToolAvailability[];
}): Promise<BaselineSnapshot | null> {
  if (fatalIssues.length > 0) {
    console.log("Skipping baseline scan because fatal current-scan errors were detected.");
    return null;
  }

  const baselineCommitSha = metrics.baseline.commitSha;
  if (metrics.baseline.status !== "generated" || !baselineCommitSha) {
    return null;
  }

  if (metrics.comparisonStatus === "input-unchanged") {
    console.log("Skipping baseline scan because scan inputs are unchanged.");
    const baselineSnapshot = createEquivalentBaselineSnapshot(metrics);
    metrics.baselineFingerprints = baselineSnapshot.fingerprints;
    metrics.trends = generateTrends(metrics, baselineSnapshot);
    console.log(`  Baseline deltas: ${metrics.trends.length} computed from equivalent inputs`);
    writeBaselineRawOutputs(rawDir, baselineSnapshot);
    return baselineSnapshot;
  }

  const baselineWorkDir = join(tmpdir(), `docnav-quality-baseline-${randomUUID()}`);
  console.log(`Materializing baseline ${baselineCommitSha.slice(0, 7)}...`);

  try {
    return await scanMaterializedBaseline({ baselineCommitSha, metrics, toolResults, rawDir, baselineWorkDir });
  } finally {
    rmSync(baselineWorkDir, { recursive: true, force: true });
  }
}

async function scanMaterializedBaseline({
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
}): Promise<BaselineSnapshot | null> {
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
    const baselineSnapshot = await scanBaselineRevision(matResult.workDir, toolResults, DEFAULT_CONFIG, {
      cacheRootDir: root,
      commitSha: baselineCommitSha
    });
    metrics.baselineFingerprints = baselineSnapshot.fingerprints;
    metrics.trends = generateTrends(metrics, baselineSnapshot);
    console.log(`  Baseline deltas: ${metrics.trends.length} computed`);
    writeBaselineRawOutputs(rawDir, baselineSnapshot);
    return baselineSnapshot;
  } catch (err: unknown) {
    console.log(`  ⚠️  Baseline scan failed: ${errorMessage(err)}`);
    metrics.baseline.status = "baseline-scan-failed";
    metrics.comparisonStatus = "baseline-unavailable";
    return null;
  }
}

function createTimings() {
  const startedAt = performance.now();
  const records: { durationMs: number; label: string }[] = [];

  const record = (label: string, startMs: number) => {
    records.push({ label, durationMs: performance.now() - startMs });
  };

  return {
    measure<T>(label: string, callback: () => T): T {
      if (!timingsEnabled) return callback();
      const startMs = performance.now();
      try {
        return callback();
      } finally {
        record(label, startMs);
      }
    },
    async measureAsync<T>(label: string, callback: () => Promise<T>): Promise<T> {
      if (!timingsEnabled) return callback();
      const startMs = performance.now();
      try {
        return await callback();
      } finally {
        record(label, startMs);
      }
    },
    print(): void {
      if (!timingsEnabled) return;
      const totalMs = performance.now() - startedAt;
      const longest = [...records].sort((a, b) => b.durationMs - a.durationMs).slice(0, 12);
      console.log("");
      console.log("Timing breakdown:");
      for (const record of longest) {
        console.log(`  ${formatTiming(record.durationMs).padStart(7)}  ${record.label}`);
      }
      console.log(`  ${formatTiming(totalMs).padStart(7)}  total`);
    }
  };
}

function formatTiming(durationMs: number): string {
  return `${durationMs.toFixed(durationMs < 100 ? 1 : 0)}ms`;
}

function createEquivalentBaselineSnapshot(metrics: QualityMetrics): BaselineSnapshot {
  return {
    fingerprints: cloneJson(metrics.currentFingerprints),
    fileMetrics: metrics.fileMetrics.map((file) => ({
      ...file,
      complexity: { ...file.complexity },
      isChanged: false
    })),
    functionMetrics: metrics.functionMetrics.map((func) => ({
      ...func,
      cyclomaticComplexity: { ...func.cyclomaticComplexity },
      isChanged: false
    })),
    duplicateCode: metrics.duplicateCode.map((fragment) => ({
      ...fragment,
      codeAreas: [...fragment.codeAreas],
      hitsChangedScope: false,
      locations: fragment.locations.map((location) => ({ ...location }))
    })),
    aggregates: cloneJson(metrics.aggregates)
  };
}

function cloneJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
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
