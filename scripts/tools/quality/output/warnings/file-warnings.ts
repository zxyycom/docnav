import type { FileMetric } from "../../model/schema.ts";
import { metricAreaWarningPolicy } from "./area-policy.ts";
import { buildMetricWarning, deltaFrom } from "./metric-warning.ts";
import type { AreaWarningPolicy, WarningCandidate, WarningContext } from "./warning-model.ts";

type FileWarningInput = {
  areaPolicy: AreaWarningPolicy;
  baseFile: FileMetric | undefined;
  context: WarningContext;
  file: FileMetric;
};

export function generateFileWarnings(files: FileMetric[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const file of files) {
    const areaPolicy = metricAreaWarningPolicy(context.config, file.codeArea);
    if (!areaPolicy) continue;

    const baseFile = context.baselineFiles.get(file.path);
    const warningInput = { areaPolicy, baseFile, context, file };
    const lineWarning = buildFileLineWarning(warningInput);
    if (lineWarning) warnings.push(lineWarning);

    const complexityWarning = buildFileComplexityWarning(warningInput);
    if (complexityWarning) warnings.push(complexityWarning);
  }

  return warnings;
}

function buildFileLineWarning(input: FileWarningInput): WarningCandidate | null {
  const { areaPolicy, baseFile, context, file } = input;
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

function buildFileComplexityWarning(input: FileWarningInput): WarningCandidate | null {
  const { areaPolicy, baseFile, context, file } = input;
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
