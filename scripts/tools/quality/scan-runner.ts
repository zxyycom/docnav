/**
 * Current revision quality scan runner.
 */

import { join } from "node:path";

import { writeQualityJsonArtifact } from "./artifacts.ts";
import { scanWithLizard } from "./tools/lizard.ts";
import { scanWithScc } from "./tools/scc.ts";
import { scanCpdPartitionsWithCache } from "./cpd-tasks.ts";
import { buildAggregates } from "./aggregate.ts";
import {
  isToolAvailable,
  normalizeFileMetrics,
  normalizeFunctionMetrics,
  selectLizardTargetFiles
} from "./scan-metrics.ts";
import type {
  CodeAreaAggregate,
  CodeAreaFileMap,
  CodeAreaFingerprint,
  DuplicateCodeFragment,
  FatalIssue,
  FileMetric,
  FunctionMetric,
  QualityConfig,
  QualityMetrics,
  ToolAvailability
} from "./schema.ts";

type ScanContext = {
  changedFiles: string[];
  config: QualityConfig;
  fatalIssues: FatalIssue[];
  fingerprints: Record<string, CodeAreaFingerprint>;
  metrics: QualityMetrics;
  rawDir: string;
  root: string;
  toolResults: ToolAvailability[];
};

export async function scanCurrentRevision({
  context,
  scanFiles,
  fileMap
}: {
  context: ScanContext;
  fileMap: CodeAreaFileMap;
  scanFiles: string[];
}): Promise<void> {
  runSccScan(context, scanFiles);
  runLizardScan(context, scanFiles);
  await runCpdScan(context, fileMap);

  context.metrics.aggregates = buildAggregates({
    fileMetrics: context.metrics.fileMetrics,
    functionMetrics: context.metrics.functionMetrics,
    duplicateCode: context.metrics.duplicateCode,
    byLanguage: context.metrics.aggregates.byLanguage,
    config: context.config
  });
}

function runSccScan(context: ScanContext, scanFiles: string[]): void {
  const { metrics, toolResults, rawDir, fatalIssues, root, config } = context;
  if (!isToolAvailable(toolResults, "scc")) return;

  console.log("Running scc...");

  const sccResult = scanWithScc({
    cwd: root,
    includePaths: scanFiles,
    excludeDirs: config.excludeDirs,
    toolConfig: config.tools.scc
  });

  if (sccResult.ok) {
    metrics.fileMetrics = normalizeFileMetrics(sccResult.files ?? [], {
      changedFiles: context.changedFiles,
      config
    });
    metrics.aggregates.byLanguage = sccResult.aggregates?.byLanguage ?? [];
    console.log(`  scc: ${metrics.fileMetrics.length} files, ${metrics.aggregates.byLanguage.length} languages`);
  } else {
    fatalIssues.push({ tool: "scc", phase: "current-scan", error: sccResult.error });
    console.log(`  ❌ scc execution/config/schema error: ${sccResult.error}`);
  }

  metrics.aggregates.byCodeArea = buildFileAreaAggregates(metrics.fileMetrics, config);
  metrics.aggregates.overall = {
    totalFiles: metrics.fileMetrics.length,
    totalLines: metrics.fileMetrics.reduce((s, f) => s + f.lines, 0),
    totalCodeLines: metrics.fileMetrics.reduce((s, f) => s + (f.codeLines || 0), 0),
    totalFunctions: 0
  };

  writeQualityJsonArtifact(join(rawDir, "scc-output.json"), metrics.fileMetrics);
}

function runLizardScan(context: ScanContext, scanFiles: string[]): void {
  const { metrics, toolResults, rawDir, fatalIssues, root, config } = context;
  if (!isToolAvailable(toolResults, "lizard")) return;

  console.log("Running Lizard...");

  const targetFiles = selectLizardTargetFiles(scanFiles, config);
  console.log(`  Lizard targets: ${targetFiles.length} files`);

  const { functions: allFunctions, errors } = scanLizardBatches({ targetFiles, root, config });
  for (const error of errors) {
    fatalIssues.push({ tool: "lizard", phase: "current-scan", error });
    console.log(`  ❌ Lizard execution/config/schema error: ${error}`);
  }

  metrics.functionMetrics = normalizeFunctionMetrics(allFunctions, {
    changedFiles: context.changedFiles,
    config
  });
  updateFunctionCounts(metrics);

  console.log(`  Lizard: ${metrics.functionMetrics.length} functions`);

  writeQualityJsonArtifact(join(rawDir, "lizard-functions.json"), metrics.functionMetrics);
}

async function runCpdScan(context: ScanContext, fileMap: CodeAreaFileMap): Promise<void> {
  const { metrics, toolResults, changedFiles, rawDir, root, config } = context;
  if (!isToolAvailable(toolResults, "pmd-cpd")) {
    console.log("  CPD not available, skipping duplicate detection");
    return;
  }

  console.log("Running PMD CPD...");

  const allFragments = await scanCpdPartitionsWithCache({
    cacheRootDir: root,
    changedFiles,
    commitSha: metrics.metadata.commitSha,
    config,
    cwd: root,
    failOnSkipped: false,
    fileMap,
    fingerprints: context.fingerprints,
    logPrefix: "  ",
    scanKind: "current",
    toolResults
  });

  metrics.duplicateCode = allFragments;
  updateDuplicateCounts(metrics, allFragments);

  console.log(`  CPD total: ${allFragments.length} duplicate fragments`);

  writeQualityJsonArtifact(join(rawDir, "cpd-fragments.json"), metrics.duplicateCode);
}

function scanLizardBatches({
  targetFiles,
  root,
  config
}: {
  config: QualityConfig;
  root: string;
  targetFiles: string[];
}): { errors: string[]; functions: FunctionMetric[] } {
  const maxFilesPerBatch = 200;
  const allFunctions: FunctionMetric[] = [];
  const errors: string[] = [];

  for (let i = 0; i < targetFiles.length; i += maxFilesPerBatch) {
    const batch = targetFiles.slice(i, i + maxFilesPerBatch);
    const batchIdx = Math.floor(i / maxFilesPerBatch) + 1;
    const totalBatches = Math.ceil(targetFiles.length / maxFilesPerBatch);

    const lizardResult = scanWithLizard({
      files: batch,
      cwd: root,
      toolConfig: config.tools.lizard
    });

    if (lizardResult.ok) {
      allFunctions.push(...(lizardResult.functions ?? []));
    } else {
      errors.push(`batch ${batchIdx}/${totalBatches}: ${lizardResult.error}`);
    }

    if (totalBatches > 1 || !lizardResult.ok) {
      console.log(`  Lizard batch ${batchIdx}/${totalBatches}: ${lizardResult.ok ? "ok" : "error"}`);
    }
  }

  return { functions: allFunctions, errors };
}

function buildFileAreaAggregates(fileMetrics: FileMetric[], config: QualityConfig): CodeAreaAggregate[] {
  const areaAggMap = new Map<string, CodeAreaAggregate>();

  for (const file of fileMetrics) {
    const existing = areaAggMap.get(file.codeArea);
    if (existing) {
      existing.files++;
      existing.lines += file.lines;
    } else {
      const areaDef = config.codeAreas[file.codeArea];
      areaAggMap.set(file.codeArea, {
        codeArea: file.codeArea,
        files: 1,
        lines: file.lines,
        functions: 0,
        warningPolicy: areaDef?.warningPolicy || "moderate"
      });
    }
  }

  return Array.from(areaAggMap.values()).sort((a, b) => b.lines - a.lines);
}

function updateFunctionCounts(metrics: QualityMetrics): void {
  const funcByArea = new Map<string, number>();
  for (const func of metrics.functionMetrics) {
    funcByArea.set(func.codeArea, (funcByArea.get(func.codeArea) || 0) + 1);
  }
  for (const agg of metrics.aggregates.byCodeArea) {
    agg.functions = funcByArea.get(agg.codeArea) || 0;
  }
  metrics.aggregates.overall.totalFunctions = metrics.functionMetrics.length;
}

function updateDuplicateCounts(metrics: QualityMetrics, allFragments: DuplicateCodeFragment[]): void {
  const dupByArea = new Map<string, number>();
  for (const dup of allFragments) {
    for (const area of dup.codeAreas) {
      dupByArea.set(area, (dupByArea.get(area) || 0) + 1);
    }
  }
  for (const agg of metrics.aggregates.byCodeArea) {
    agg.duplicateFragments = dupByArea.get(agg.codeArea) || 0;
  }
}
