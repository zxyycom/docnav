#!/usr/bin/env bun

/**
 * Docnav 代码质量观测命令入口。
 *
 * 代码质量扫描：Clippy 保持 Rust 阻断 gate，Lizard/scc/PMD CPD
 * 生成代码质量快照、warning 状态和报告。
 */

import { resolve, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

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
import { detectScanInputChange } from "../tools/quality/input/revisions.ts";
import { generateWarningChannels } from "../tools/quality/output/warnings/generator.ts";
import {
  collectScanFiles,
  buildFingerprints
} from "../tools/quality/input/files.ts";
import { runCurrentRevisionScan } from "../tools/quality/measurement/current-revision/index.ts";
import type { ChangeScope, QualityScanOptions } from "../tools/quality/scan-command/index.ts";
import {
  parseArgs,
  prepareArtifactDirs,
  initializeToolResults,
  collectToolMetadata,
  configureBaseline,
  setComparisonStatus,
  resolveChangedFilesForScan,
  writeArtifacts,
  printSummary,
  printWarningStatus,
  qualityCheckStatus,
  validateOutput,
  logFingerprints,
  formatFatalIssue,
  getGitSha,
  getGitCommitTitle,
  createTimings,
  maybeScanBaselineRevision,
  type Timings
} from "../tools/quality/scan-command/index.ts";

export { runBaselineRevisionScan } from "../tools/quality/measurement/baseline-revision.ts";
export { buildAggregates } from "../tools/quality/measurement/aggregate.ts";
export { generateTrends } from "../tools/quality/output/trends.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

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

  printBanner(opts.scanProfile);

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

  const baselineSnapshot = await timings.measureAsync("baseline snapshot", () =>
    maybeScanBaselineRevision({ config: DEFAULT_CONFIG, root, runtime })
  );
  generateWarnings(runtime.metrics, changedInput.inputScope, baselineSnapshot, opts.scanProfile, timings);
  finishScan({ artifactDir, runtime, timings });
}

function printBanner(scanProfile: QualityScanOptions["scanProfile"]): void {
  console.log("Docnav Code Quality Observability");
  console.log(`Profile: ${scanProfile}`);
  if (scanProfile === "quick") {
    console.log("Quick check — skips baseline comparison and PMD CPD duplicate detection.");
  } else {
    console.log("Full check — runs all configured scanners; baseline comparison is opt-in.");
  }
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
    fileMap: inputs.fileMap,
    scanProfile: runtime.opts.scanProfile
  }));
}

function generateWarnings(
  metrics: QualityMetrics,
  scope: ChangeScope,
  baselineSnapshot: BaselineSnapshot | null,
  scanProfile: QualityScanOptions["scanProfile"],
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
    comparisonStatus: metrics.comparisonStatus,
    validateAcceptedWarnings: scanProfile === "full"
  }));
  const warningCounts = [
    `all=${metrics.warnings.all.length}`,
    `changed=${metrics.warnings.changed.length}`,
    `regressions=${metrics.warnings.regressions.length}`,
    `withAcceptedReason=${metrics.warnings.all.filter((warning) => warning.acceptedReason).length}`
  ].join(", ");
  console.log(`  Warning records generated: ${warningCounts}`);
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
  recordValidationIssues(fatalIssues, validation.errors);

  if (fatalIssues.length > 0) {
    finishFatalScan(artifactDir, fatalIssues);
    return;
  }

  timings.measure("print warning status", () =>
    printWarningStatus({
      artifactDir,
      metrics,
      scanProfile: opts.scanProfile,
      verificationOutput: opts.verificationOutput
    })
  );
  printSuccessfulScanCompletion(qualityCheckStatus(metrics), artifactDir);

  timings.print();
  process.exit(0);
}

function recordValidationIssues(fatalIssues: FatalIssue[], validationErrors: string[]): void {
  if (validationErrors.length === 0) {
    return;
  }
  fatalIssues.push({
    tool: "metrics",
    phase: "validation",
    error: validationErrors.join("; ")
  });
}

function finishFatalScan(artifactDir: string, fatalIssues: FatalIssue[]): void {
  console.log("");
  console.log("❌ Quality scan failed.");
  console.log(`Artifacts in: ${artifactDir}/`);
  console.error("Fatal quality scan issues:");
  for (const issue of fatalIssues) {
    console.error(`  - ${formatFatalIssue(issue)}`);
  }
  process.exit(2);
}

function printSuccessfulScanCompletion(status: "passed" | "warning", artifactDir: string): void {
  console.log("");
  console.log(successfulScanMessage(status));
  console.log(`Artifacts in: ${artifactDir}/`);
}

function successfulScanMessage(status: "passed" | "warning"): string {
  if (status === "warning") {
    return "⚠️ Quality scan complete with warnings.";
  }
  return "✅ Quality scan complete.";
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
