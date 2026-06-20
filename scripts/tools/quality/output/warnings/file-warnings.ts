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

type FileWarningBuilder = (input: FileWarningInput) => WarningCandidate | null;

const FILE_WARNING_BUILDERS: FileWarningBuilder[] = [
  buildFileLineWarning,
  buildFileDecisionTokenWarning
];

export function generateFileWarnings(files: FileMetric[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const file of files) {
    const areaPolicy = metricAreaWarningPolicy(context.config, file.codeArea);
    if (!areaPolicy) continue;

    const baseFile = context.baselineFiles.get(file.path);
    const warningInput = { areaPolicy, baseFile, context, file };
    for (const buildWarning of FILE_WARNING_BUILDERS) {
      const warning = buildWarning(warningInput);
      if (warning) warnings.push(warning);
    }
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

function buildFileDecisionTokenWarning(input: FileWarningInput): WarningCandidate | null {
  const { areaPolicy, baseFile, context, file } = input;
  const decisionTokenFloor = context.config.scc?.fileComplexity?.absoluteFloor ?? 20;
  const decisionTokenDelta = context.config.scc?.fileComplexity?.changedDelta ?? 10;
  const baselineDecisionTokens = baseFile?.complexity?.value ?? (context.hasBaselineFiles ? 0 : null);
  const decisionTokenCount = file.complexity.value;
  const decisionTokenDeltaValue = deltaFrom(decisionTokenCount, baselineDecisionTokens);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineDecisionTokens,
    codeArea: file.codeArea,
    deltaFloor: decisionTokenDelta,
    deltaValue: decisionTokenDeltaValue,
    floor: decisionTokenFloor,
    isChanged: file.isChanged,
    line: null,
    message: `File "${file.path}" has ${decisionTokenCount} scc decision tokens (threshold: ${decisionTokenFloor} decision tokens)`,
    metric: "decision-tokens",
    path: file.path,
    ruleId: "scc-file-decision-tokens",
    sourceTool: "scc",
    suggestion: "Review with lizard function CC and file responsibility before refactoring",
    value: decisionTokenCount
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
