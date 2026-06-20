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

import { DEFAULT_CONFIG } from "../tools/quality/model/config.ts";
import { createEmptyMetrics } from "../tools/quality/model/schema.ts";
import type {
  BaselineSnapshot,
  CodeAreaFileMap,
  CodeAreaFingerprint,
  FatalIssue,
  QualityMetrics,
  ToolAvailability
} from "../tools/quality/model/schema.ts";
import { errorMessage } from "../tools/errors.ts";
import { classifyFiles } from "../tools/quality/model/code-areas.ts";
import {
  materializeBaselineRevision,
  detectScanInputChange
} from "../tools/quality/input/revisions.ts";
import { generateWarningChannels } from "../tools/quality/output/warnings/generator.ts";
import {
  collectScanFiles,
  buildFingerprints
} from "../tools/quality/input/files.ts";
import { runCurrentRevisionScan } from "../tools/quality/measurement/current-revision/index.ts";
import { runBaselineRevisionScan } from "../tools/quality/measurement/baseline-revision.ts";
import { generateTrends } from "../tools/quality/output/trends.ts";
import type { ChangeScope, QualityScanOptions } from "../tools/quality/scan-command/index.ts";
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
} from "../tools/quality/scan-command/index.ts";

export { runBaselineRevisionScan } from "../tools/quality/measurement/baseline-revision.ts";
export { buildAggregates } from "../tools/quality/measurement/aggregate.ts";
export { generateTrends } from "../tools/quality/output/trends.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const timingsEnabled = process.env.DOCNAV_QUALITY_TIMINGS === "1";

type Timings = ReturnType<typeof createTimings>;

type ScanInputs = {
  fileMap: CodeAreaFileMap;
  fingerprints: Record<string, CodeAreaFingerprint>;
  scanFiles: string[];
};

type ChangedInputScope = {
  changedFiles: string[];
  inputScope: ChangeScope;
};

type RuntimeContext = {
  fatalIssues: FatalIssue[];
  metrics: QualityMetrics;
  opts: QualityScanOptions;
  rawDir: string;
  toolResults: ToolAvailability[];
};

async function main() {
  const timings = createTimings();
  const opts = parseArgs();

  printBanner();

  const artifactDir = resolve(root, opts.artifactDir);
  const { rawDir } = timings.measure("prepare artifact dirs", () => prepareArtifactDirs(artifactDir));
  const runtime = await prepareRuntimeContext({ opts, rawDir, timings });
  const inputs = collectScanInputs(timings);
  attachFingerprints(runtime.metrics, inputs.fingerprints);

  timings.measure("configure baseline", () => configureBaseline({
    metrics: runtime.metrics,
    opts,
    tools: runtime.metrics.metadata.tools,
    root
  }));

  const changedInput = detectChangedInputScope(runtime.metrics, opts, timings);
  await scanCurrentRevision(runtime, inputs, changedInput.changedFiles, timings);
  timings.measure("set comparison status", () => setComparisonStatus(runtime.metrics, changedInput.inputScope));

  const baselineSnapshot = await timings.measureAsync("baseline snapshot", () => maybeScanBaseline(runtime));
  generateWarnings(runtime.metrics, changedInput.inputScope, baselineSnapshot, timings);
  finishScan({ artifactDir, runtime, timings });
}

function printBanner(): void {
  console.log("Docnav Code Quality Observability");
  console.log("Non-blocking snapshot — metric values do not cause failure.");
  console.log("");
}

async function prepareRuntimeContext({
  opts,
  rawDir,
  timings
}: {
  opts: QualityScanOptions;
  rawDir: string;
  timings: Timings;
}): Promise<RuntimeContext> {
  const commitSha = timings.measure("git rev-parse HEAD", () => getGitSha(root));
  const commitTitle = timings.measure("git commit title", () => getGitCommitTitle(commitSha, root));
  const toolResults = await timings.measureAsync("tool availability", () => initializeToolResults(root));
  const tools = timings.measure("tool metadata", () => collectToolMetadata(toolResults));
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

  return { fatalIssues: [], metrics, opts, rawDir, toolResults };
}

function collectScanInputs(timings: Timings): ScanInputs {
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

  return { fileMap, fingerprints, scanFiles };
}

function attachFingerprints(
  metrics: QualityMetrics,
  fingerprints: Record<string, CodeAreaFingerprint>
): void {
  metrics.currentFingerprints = fingerprints;
}

function detectChangedInputScope(
  metrics: QualityMetrics,
  opts: QualityScanOptions,
  timings: Timings
): ChangedInputScope {
  const inputScope = timings.measure("detect changed scan inputs", () => detectScanInputChange({
    baselineSha: metrics.baseline.commitSha,
    cwd: root,
    scanInputPaths: DEFAULT_CONFIG.include
  }));
  const changedFiles = timings.measure("resolve changed files", () =>
    resolveChangedFilesForScan({ opts, root, scope: inputScope })
  );
  console.log(`  Changed files in scan scope: ${changedFiles.length}`);
  return { changedFiles, inputScope };
}

async function scanCurrentRevision(
  runtime: RuntimeContext,
  inputs: ScanInputs,
  changedFiles: string[],
  timings: Timings
): Promise<void> {
  await timings.measureAsync("scan current revision", () => runCurrentRevisionScan({
    context: {
      metrics: runtime.metrics,
      toolResults: runtime.toolResults,
      changedFiles,
      rawDir: runtime.rawDir,
      fatalIssues: runtime.fatalIssues,
      root,
      fingerprints: inputs.fingerprints,
      config: DEFAULT_CONFIG
    },
    scanFiles: inputs.scanFiles,
    fileMap: inputs.fileMap
  }));
}

function generateWarnings(
  metrics: QualityMetrics,
  scope: ChangeScope,
  baselineSnapshot: BaselineSnapshot | null,
  timings: Timings
): void {
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
}

function finishScan({
  artifactDir,
  runtime,
  timings
}: {
  artifactDir: string;
  runtime: RuntimeContext;
  timings: Timings;
}): void {
  const { fatalIssues, metrics, opts } = runtime;

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
}: RuntimeContext): Promise<BaselineSnapshot | null> {
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
  const matResult = materializeBaselineRevision({
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
    const baselineSnapshot = await runBaselineRevisionScan(matResult.workDir, toolResults, DEFAULT_CONFIG, {
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
