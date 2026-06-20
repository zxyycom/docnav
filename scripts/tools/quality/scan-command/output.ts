import { mkdirSync } from "node:fs";
import { join } from "node:path";

import { DEFAULT_CONFIG } from "../model/config.ts";
import { validateMetrics } from "../model/schema.ts";
import { writeTextFile } from "../../fs.ts";
import { toNdjson } from "../../ndjson.ts";
import { writeQualityJsonArtifact } from "../output/artifacts.ts";
import { generateMarkdownReport } from "../output/report/index.ts";
import type {
  BaselineSnapshot,
  CodeAreaFingerprint,
  FatalIssue,
  QualityMetrics
} from "../model/schema.ts";

export function prepareArtifactDirs(artifactDir: string): { rawDir: string } {
  const rawDir = join(artifactDir, "raw");
  mkdirSync(artifactDir, { recursive: true });
  mkdirSync(rawDir, { recursive: true });
  return { rawDir };
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
