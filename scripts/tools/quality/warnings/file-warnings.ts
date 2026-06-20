import type { FileMetric } from "../schema.ts";
import { metricAreaWarningPolicy } from "./policy.ts";
import { buildMetricWarning, deltaFrom } from "./record.ts";
import type { AreaWarningPolicy, WarningCandidate, WarningContext } from "./types.ts";

export function generateFileWarnings(files: FileMetric[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const file of files) {
    const areaPolicy = metricAreaWarningPolicy(context.config, file.codeArea);
    if (!areaPolicy) continue;

    const baseFile = context.baselineFiles.get(file.path);
    const lineWarning = buildFileLineWarning(file, baseFile, context, areaPolicy);
    if (lineWarning) warnings.push(lineWarning);

    const complexityWarning = buildFileComplexityWarning(file, baseFile, context, areaPolicy);
    if (complexityWarning) warnings.push(complexityWarning);
  }

  return warnings;
}

function buildFileLineWarning(
  file: FileMetric,
  baseFile: FileMetric | undefined,
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const lineFloor = context.config.scc?.fileCodeLines?.absoluteFloor ?? 300;
  const lineDelta = context.config.scc?.fileCodeLines?.changedDelta ?? 100;
  const fileCodeLines = file.codeLines ?? null;
  const baselineCodeLines = baselineFileCodeLines(baseFile, context.hasBaselineFiles);
  const lineDeltaValue = deltaFrom(fileCodeLines, baselineCodeLines);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineCodeLines,
    codeArea: file.codeArea,
    deltaFloor: lineDelta,
    deltaValue: lineDeltaValue,
    floor: lineFloor,
    isChanged: file.isChanged,
    line: null,
    message: `File "${file.path}" has ${fileCodeLines} code lines (threshold: ${lineFloor} code lines)`,
    metric: "code-lines",
    path: file.path,
    ruleId: "scc-file-code-lines",
    sourceTool: "scc",
    suggestion: fileLineSuggestion(fileCodeLines, lineFloor),
    value: fileCodeLines
  });
}

function buildFileComplexityWarning(
  file: FileMetric,
  baseFile: FileMetric | undefined,
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const ccFloor = context.config.scc?.fileComplexity?.absoluteFloor ?? 20;
  const ccDelta = context.config.scc?.fileComplexity?.changedDelta ?? 10;
  const baseComplexity = baseFile?.complexity?.value ?? (context.hasBaselineFiles ? 0 : null);
  const fileComplexity = file.complexity.value;
  const ccDeltaValue = deltaFrom(fileComplexity, baseComplexity);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baseComplexity,
    codeArea: file.codeArea,
    deltaFloor: ccDelta,
    deltaValue: ccDeltaValue,
    floor: ccFloor,
    isChanged: file.isChanged,
    line: null,
    message: `File "${file.path}" has complexity ${fileComplexity} (threshold: ${ccFloor} complexity)`,
    metric: "complexity",
    path: file.path,
    ruleId: "scc-file-complexity",
    sourceTool: "scc",
    suggestion: "Consider splitting complex logic into smaller functions",
    value: fileComplexity
  });
}

function baselineFileCodeLines(
  baseFile: FileMetric | undefined,
  hasBaselineFiles: boolean
): number | null {
  if (baseFile) {
    return baseFile.codeLines ?? null;
  }
  return hasBaselineFiles ? 0 : null;
}

function fileLineSuggestion(codeLines: number | null, floor: number): string {
  if (codeLines !== null && codeLines > floor * 3) {
    return "Consider splitting this file into smaller modules";
  }
  return "Review if the file can be refactored";
}
