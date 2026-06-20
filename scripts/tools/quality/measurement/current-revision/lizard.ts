import { join } from "node:path";

import { writeQualityJsonArtifact } from "../../output/artifacts.ts";
import { scanWithLizard } from "../scanners/lizard.ts";
import {
  isToolAvailable,
  normalizeFunctionMetrics,
  selectLizardTargetFiles
} from "../metrics.ts";
import type { ScanContext } from "./scan-context.ts";
import type { FunctionMetric, QualityConfig, QualityMetrics } from "../../model/schema.ts";

export function runLizardScan(context: ScanContext, scanFiles: string[]): void {
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
