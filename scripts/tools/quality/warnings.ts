/**
 * Warning 规则生成器。
 *
 * 使用当前快照、可选 changed scope、可选 baseline delta 和绝对下限组合
 * 生成 warning records。默认聚焦全量观测报告；只有启用 comparison 时
 * 才生成 changed/regression CI annotation 输入。
 */

import type {
  DuplicateCodeFragment,
  FileMetric,
  FunctionMetric,
  QualityConfig,
  WarningChannels,
  WarningRecord
} from "./schema.ts";

type WarningBaseline = {
  duplicates: DuplicateCodeFragment[];
  files: FileMetric[];
  functions: FunctionMetric[];
} | null;

type GenerateWarningsParams = {
  baseline: WarningBaseline;
  comparisonStatus: string;
  config: QualityConfig;
  duplicates: DuplicateCodeFragment[];
  files: FileMetric[];
  functions: FunctionMetric[];
  scope: { changed: boolean; changedFiles: string[] };
};

type AreaWarningPolicy = {
  isWatchlistOnly: boolean;
  level: "info" | "warning";
};

type MetricWarningSpec = {
  areaPolicy: AreaWarningPolicy;
  baselineValue: number | null;
  codeArea: string;
  deltaFloor: number;
  deltaValue: number | null;
  floor: number;
  isChanged: boolean;
  line: number | null;
  message: string;
  metric: string;
  path: string;
  ruleId: string;
  sourceTool: string;
  suggestion: string;
  value: number | null;
};

type WarningContext = {
  baselineDuplicateIndex: Map<string, number>;
  baselineFiles: Map<string, FileMetric>;
  baselineFunctions: Map<string, FunctionMetric>;
  config: QualityConfig;
  hasBaselineDuplicates: boolean;
  hasBaselineFiles: boolean;
  hasBaselineFunctions: boolean;
};

type WarningCandidate = {
  deltaFloor: number;
  isWatchlistOnly: boolean;
  record: WarningRecord;
};

export function generateWarningChannels(params: GenerateWarningsParams): WarningChannels {
  const { files, functions, duplicates, config, baseline, comparisonStatus } = params;

  const context = buildWarningContext(config, baseline);
  const candidates = [
    ...generateFileWarnings(files, context),
    ...generateFunctionWarnings(functions, context),
    ...generateDuplicateWarnings(duplicates, context)
  ];

  candidates.sort((a, b) => compareWarnings(a.record, b.record));

  const all = candidates.map((candidate) => candidate.record);
  const changedCandidates = suppressesChangedWarnings(comparisonStatus)
    ? []
    : candidates.filter(shouldEmitChangedWarning);

  return {
    all,
    changed: changedCandidates.map((candidate) => candidate.record),
    regressions: changedCandidates
      .filter((candidate) => candidate.record.deltaValue !== null && candidate.record.deltaValue > candidate.deltaFloor)
      .map((candidate) => candidate.record)
  };
}

export function generateWarnings(params: GenerateWarningsParams): WarningRecord[] {
  return generateWarningChannels(params).changed;
}

function suppressesChangedWarnings(comparisonStatus: string): boolean {
  return comparisonStatus === "input-unchanged" || comparisonStatus === "baseline-unavailable";
}

function buildWarningContext(config: QualityConfig, baseline: WarningBaseline): WarningContext {
  const baselineFiles = buildFileBaselineMap(baseline?.files || []);
  const baselineFunctions = buildFunctionBaselineMap(baseline?.functions || []);
  const baselineDuplicateIndex = buildDuplicateBaselineIndex(baseline?.duplicates || []);
  const hasBaselineFiles = Array.isArray(baseline?.files);
  const hasBaselineFunctions = Array.isArray(baseline?.functions);
  const hasBaselineDuplicates = Array.isArray(baseline?.duplicates);

  return {
    baselineDuplicateIndex,
    baselineFiles,
    baselineFunctions,
    config,
    hasBaselineDuplicates,
    hasBaselineFiles,
    hasBaselineFunctions
  };
}

function generateFileWarnings(files: FileMetric[], context: WarningContext): WarningCandidate[] {
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

function buildMetricWarning(spec: MetricWarningSpec): WarningCandidate | null {
  if (spec.value === null || !exceedsAbsoluteFloor(spec.value, spec.floor)) {
    return null;
  }

  return {
    deltaFloor: spec.deltaFloor,
    isWatchlistOnly: spec.areaPolicy.isWatchlistOnly,
    record: {
      level: spec.areaPolicy.level,
      ruleId: spec.ruleId,
      sourceTool: spec.sourceTool,
      path: spec.path,
      line: spec.line,
      codeArea: spec.codeArea,
      metric: spec.metric,
      value: spec.value,
      comparisonBasis: basisFor(spec.isChanged, spec.deltaValue),
      baselineValue: spec.baselineValue,
      deltaValue: spec.deltaValue,
      isChanged: spec.isChanged,
      message: spec.message,
      suggestion: spec.suggestion
    }
  };
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

function generateFunctionWarnings(functions: FunctionMetric[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const func of functions) {
    const areaPolicy = metricAreaWarningPolicy(context.config, func.codeArea);
    if (!areaPolicy) continue;

    const baselineFunc = context.baselineFunctions.get(functionKey(func));
    const complexityWarning = buildFunctionComplexityWarning(func, baselineFunc, context, areaPolicy);
    if (complexityWarning) warnings.push(complexityWarning);

    const lineWarning = buildFunctionLineWarning(func, baselineFunc, context, areaPolicy);
    if (lineWarning) warnings.push(lineWarning);

    const parameterWarning = buildFunctionParameterWarning(func, baselineFunc, context, areaPolicy);
    if (parameterWarning) warnings.push(parameterWarning);
  }

  return warnings;
}

function buildFunctionComplexityWarning(
  func: FunctionMetric,
  baselineFunc: FunctionMetric | undefined,
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const ccFloor = context.config.lizard?.cyclomaticComplexity?.absoluteFloor ?? 10;
  const ccDelta = context.config.lizard?.cyclomaticComplexity?.changedDelta ?? 5;
  const baselineCc = baselineFunc?.cyclomaticComplexity?.value ?? (context.hasBaselineFunctions ? 0 : null);
  const functionComplexity = func.cyclomaticComplexity.value;
  const ccDeltaValue = deltaFrom(functionComplexity, baselineCc);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineCc,
    codeArea: func.codeArea,
    deltaFloor: ccDelta,
    deltaValue: ccDeltaValue,
    floor: ccFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has cyclomatic complexity ${functionComplexity} (threshold: ${ccFloor} CC)`,
    metric: "cyclomatic-complexity",
    path: func.file,
    ruleId: "lizard-cyclomatic-complexity",
    sourceTool: "lizard",
    suggestion: "Consider breaking this function into smaller, more focused functions",
    value: functionComplexity
  });
}

function buildFunctionLineWarning(
  func: FunctionMetric,
  baselineFunc: FunctionMetric | undefined,
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const lineFloor = context.config.lizard?.functionCodeLines?.absoluteFloor ?? 50;
  const lineDeltaCfg = context.config.lizard?.functionCodeLines?.changedDelta ?? 20;
  const baselineFunctionLines = baselineFunc?.lines ?? (context.hasBaselineFunctions ? 0 : null);
  const functionLineDelta = deltaFrom(func.lines, baselineFunctionLines);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineFunctionLines,
    codeArea: func.codeArea,
    deltaFloor: lineDeltaCfg,
    deltaValue: functionLineDelta,
    floor: lineFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.lines} code lines (Lizard NLOC; threshold: ${lineFloor} code lines)`,
    metric: "function-code-lines",
    path: func.file,
    ruleId: "lizard-function-code-lines",
    sourceTool: "lizard",
    suggestion: "Consider extracting parts of this function into separate functions",
    value: func.lines
  });
}

function buildFunctionParameterWarning(
  func: FunctionMetric,
  baselineFunc: FunctionMetric | undefined,
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const paramFloor = context.config.lizard?.parameterCount?.absoluteFloor ?? 5;
  const paramDeltaCfg = context.config.lizard?.parameterCount?.changedDelta ?? 2;
  const baselineParameterCount = baselineFunc?.parameterCount ?? (context.hasBaselineFunctions ? 0 : null);
  const paramDeltaValue = deltaFrom(func.parameterCount, baselineParameterCount);

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineParameterCount,
    codeArea: func.codeArea,
    deltaFloor: paramDeltaCfg,
    deltaValue: paramDeltaValue,
    floor: paramFloor,
    isChanged: func.isChanged,
    line: func.startLine,
    message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.parameterCount} parameters (threshold: ${paramFloor} parameters)`,
    metric: "parameter-count",
    path: func.file,
    ruleId: "lizard-parameter-count",
    sourceTool: "lizard",
    suggestion: "Consider using a parameter object or splitting the function",
    value: func.parameterCount
  });
}

function generateDuplicateWarnings(duplicates: DuplicateCodeFragment[], context: WarningContext): WarningCandidate[] {
  const warnings: WarningCandidate[] = [];

  for (const dup of duplicates) {
    const uniqueAreas = duplicateCodeAreas(dup);
    const areaPolicy = duplicateAreaWarningPolicy(uniqueAreas, context.config);
    if (!areaPolicy) continue;

    const warning = buildDuplicateWarning(dup, uniqueAreas, context, areaPolicy);
    if (warning) warnings.push(warning);
  }

  return warnings;
}

function buildDuplicateWarning(
  dup: DuplicateCodeFragment,
  uniqueAreas: string[],
  context: WarningContext,
  areaPolicy: AreaWarningPolicy
): WarningCandidate | null {
  const primaryLocation = dup.locations[0];
  const baselineDuplicateCount = countMatchingBaselineDuplicates(
    dup,
    context.baselineDuplicateIndex,
    context.hasBaselineDuplicates
  );
  const duplicateDelta = baselineDuplicateCount === null ? null : 1 - baselineDuplicateCount;
  const duplicateDeltaFloor = context.config.pmdCpd?.duplicateFragments?.changedDelta ?? 0;
  const locations = dup.locations.map(formatDuplicateWarningLocation).join(", ");

  return buildMetricWarning({
    areaPolicy,
    baselineValue: baselineDuplicateCount,
    codeArea: uniqueAreas.join(",") || "unknown",
    deltaFloor: duplicateDeltaFloor,
    deltaValue: duplicateDelta,
    floor: 0,
    isChanged: dup.hitsChangedScope,
    line: primaryLocation?.startLine ?? null,
    message: `Duplicate code fragment (${dup.tokenCount} tokens) across ${dup.locations.length} locations in areas [${uniqueAreas.join(", ")}]`,
    metric: "duplicate-tokens",
    path: primaryLocation?.path || "unknown",
    ruleId: "pmd-cpd-duplicate-code",
    sourceTool: "pmd-cpd",
    suggestion: `Consider extracting shared code into a common function or module. Locations: ${locations}`,
    value: dup.tokenCount
  });
}

function metricAreaWarningPolicy(config: QualityConfig, codeArea: string): AreaWarningPolicy | null {
  const areaConfig = config.codeAreas[codeArea];
  if (!areaConfig) return null;
  if (areaConfig.warningPolicy === "exclude-warnings") return null;

  const isWatchlistOnly = areaConfig.warningPolicy === "watchlist-only";
  return {
    isWatchlistOnly,
    level: isWatchlistOnly ? "info" : "warning"
  };
}

function duplicateAreaWarningPolicy(uniqueAreas: string[], config: QualityConfig): AreaWarningPolicy | null {
  if (uniqueAreas.length > 0 && uniqueAreas.every((area) => codeAreaHasPolicy(config, area, "exclude-warnings"))) {
    return null;
  }

  const isWatchlistOnly = uniqueAreas.length > 0
    && uniqueAreas.every((area) => codeAreaHasPolicy(config, area, "watchlist-only"));

  return {
    isWatchlistOnly,
    level: isWatchlistOnly ? "info" : "warning"
  };
}

function codeAreaHasPolicy(config: QualityConfig, codeArea: string, warningPolicy: string): boolean {
  const areaConfig = config.codeAreas[codeArea];
  return Boolean(areaConfig && areaConfig.warningPolicy === warningPolicy);
}

function duplicateCodeAreas(dup: DuplicateCodeFragment): string[] {
  const involvedAreas = dup.locations.map((location) => location.codeArea).filter(Boolean);
  return [...new Set(involvedAreas)] as string[];
}

function formatDuplicateWarningLocation(location: DuplicateCodeFragment["locations"][number]): string {
  return `${location.path}:${location.startLine}`;
}

function compareWarnings(a: WarningRecord, b: WarningRecord): number {
  const lvlOrder: Record<string, number> = { error: 0, warning: 1, info: 2 };
  const lvlDiff = lvlOrder[a.level] - lvlOrder[b.level];
  if (lvlDiff !== 0) return lvlDiff;
  return b.value - a.value;
}

function shouldEmitChangedWarning(candidate: WarningCandidate): boolean {
  const { record, isWatchlistOnly, deltaFloor } = candidate;
  if (!record.isChanged) return false;
  if (isWatchlistOnly) return true;
  if (record.deltaValue === null || record.deltaValue === undefined) return true;
  return record.deltaValue > deltaFloor;
}

function exceedsAbsoluteFloor(value: number | null | undefined, floor: number): boolean {
  return value !== null && value !== undefined && value > floor;
}

function basisFor(isChanged: boolean, delta: number | null): string {
  if (isChanged && delta !== null && delta !== undefined) {
    return "delta";
  }
  return isChanged ? "changed-scope" : "absolute";
}

function deltaFrom(current: number | null | undefined, baseline: number | null | undefined): number | null {
  if (current === null || current === undefined || baseline === null || baseline === undefined) {
    return null;
  }
  return current - baseline;
}

function buildFileBaselineMap(files: FileMetric[]): Map<string, FileMetric> {
  return new Map(files.map((file) => [file.path, file]));
}

function buildFunctionBaselineMap(functions: FunctionMetric[]): Map<string, FunctionMetric> {
  return new Map(functions.map((func) => [functionKey(func), func]));
}

function functionKey(func: FunctionMetric): string {
  return `${func.file}:${func.name}:${func.startLine}`;
}

function buildDuplicateBaselineIndex(duplicates: DuplicateCodeFragment[]): Map<string, number> {
  const index = new Map<string, number>();
  for (const dup of duplicates) {
    const key = duplicateKey(dup);
    index.set(key, (index.get(key) || 0) + 1);
  }
  return index;
}

function countMatchingBaselineDuplicates(
  dup: DuplicateCodeFragment,
  baselineIndex: Map<string, number>,
  hasBaselineDuplicates: boolean
): number | null {
  if (!hasBaselineDuplicates) {
    return null;
  }
  return baselineIndex.get(duplicateKey(dup)) || 0;
}

function duplicateKey(dup: DuplicateCodeFragment): string {
  return dup.locations
    .map((loc) => `${loc.path}:${loc.startLine}`)
    .sort()
    .join("|");
}
