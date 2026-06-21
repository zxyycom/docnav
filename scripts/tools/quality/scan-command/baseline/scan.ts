import { rmSync } from "node:fs";
import { randomUUID } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { errorMessage } from "../../../errors.ts";
import { materializeBaselineRevision } from "../../input/revisions.ts";
import { runBaselineRevisionScan } from "../../measurement/baseline-revision.ts";
import { generateTrends } from "../../output/trends.ts";
import type {
  BaselineSnapshot,
  FatalIssue,
  QualityConfig,
  QualityMetrics,
  ToolAvailability
} from "../../model/schema.ts";
import { writeBaselineRawOutputs } from "../command-output.ts";

export type BaselineScanRuntime = {
  fatalIssues: FatalIssue[];
  metrics: QualityMetrics;
  rawDir: string;
  toolResults: ToolAvailability[];
};

type MaterializedBaselineScanOptions = {
  baselineCommitSha: string;
  baselineWorkDir: string;
  config: QualityConfig;
  metrics: QualityMetrics;
  rawDir: string;
  root: string;
  toolResults: ToolAvailability[];
};

export async function maybeScanBaselineRevision({
  config,
  root,
  runtime
}: {
  config: QualityConfig;
  root: string;
  runtime: BaselineScanRuntime;
}): Promise<BaselineSnapshot | null> {
  const { metrics, toolResults, rawDir, fatalIssues } = runtime;
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
    return await scanMaterializedBaseline({
      baselineCommitSha,
      baselineWorkDir,
      config,
      metrics,
      rawDir,
      root,
      toolResults
    });
  } finally {
    rmSync(baselineWorkDir, { recursive: true, force: true });
  }
}

async function scanMaterializedBaseline(options: MaterializedBaselineScanOptions): Promise<BaselineSnapshot | null> {
  const {
    baselineCommitSha,
    baselineWorkDir,
    config,
    metrics,
    rawDir,
    root,
    toolResults
  } = options;

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
    const baselineSnapshot = await runBaselineRevisionScan(matResult.workDir, toolResults, config, {
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

function createEquivalentBaselineSnapshot(metrics: QualityMetrics): BaselineSnapshot {
  return {
    fingerprints: cloneJson(metrics.currentFingerprints),
    fileMetrics: metrics.fileMetrics.map((file) => ({
      ...file,
      decisionTokens: { ...file.decisionTokens },
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
