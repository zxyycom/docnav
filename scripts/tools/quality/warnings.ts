/**
 * Warning 规则生成器。
 *
 * 使用 changed scope、previous-code baseline delta 和绝对下限组合
 * 生成 warning records。优先指向 changed files 或 changed functions，
 * 避免对未改动历史热点或 text-only 变更默认刷 CI annotation。
 *
 * 来源：openspec/changes/implement-code-quality-observability/tasks.md task 4.3
 */

/**
 * @typedef {import('./schema.ts').FileMetric} FileMetric
 * @typedef {import('./schema.ts').FunctionMetric} FunctionMetric
 * @typedef {import('./schema.ts').DuplicateCodeFragment} DuplicateCodeFragment
 * @typedef {import('./schema.ts').WarningRecord} WarningRecord
 * @typedef {import('./config.ts').DEFAULT_CONFIG} QualityConfig
 */

/**
 * 生成 warning records。
 *
 * @param {object} params
 * @param {FileMetric[]} params.files - 当前文件指标
 * @param {FunctionMetric[]} params.functions - 当前函数指标
 * @param {DuplicateCodeFragment[]} params.duplicates - 当前重复代码片段
 * @param {QualityConfig} params.config - 质量观测配置
 * @param {{ changed: boolean, changedFiles: string[] }} params.scope - Change scope
 * @param {object} [params.baseline] - Baseline metrics used to compute deltas
 * @param {FileMetric[]} [params.baseline.files]
 * @param {FunctionMetric[]} [params.baseline.functions]
 * @param {DuplicateCodeFragment[]} [params.baseline.duplicates]
 * @param {string} params.comparisonStatus - compared, input-unchanged, baseline-unavailable
 * @returns {WarningRecord[]}
 */
export function generateWarnings({ files, functions, duplicates, config, baseline, comparisonStatus }: ExternalValue) {
  /** @type {WarningRecord[]} */
  const warnings: ExternalValue[] = [];

  // 当 comparison status 为 input-unchanged 或 baseline-unavailable 时，
  // 不生成复杂度或重复代码 annotation（CI annotation 降级为 summary/watchlist）
  if (comparisonStatus === "input-unchanged" || comparisonStatus === "baseline-unavailable") {
    // 仍可以生成 info-level watchlist entries
    return [];
  }

  const baselineFiles = buildFileBaselineMap(baseline?.files || []);
  const baselineFunctions = buildFunctionBaselineMap(baseline?.functions || []);
  const baselineDuplicateIndex = buildDuplicateBaselineIndex(baseline?.duplicates || []);
  const hasBaselineFiles = Array.isArray(baseline?.files);
  const hasBaselineFunctions = Array.isArray(baseline?.functions);
  const hasBaselineDuplicates = Array.isArray(baseline?.duplicates);

  // File-level warnings (from scc data)
  for (const file of files) {
    const areaConfig = config.codeAreas[file.codeArea];
    if (!areaConfig) continue;
    if (areaConfig.warningPolicy === "exclude-warnings") continue;

    const isWatchlistOnly = areaConfig.warningPolicy === "watchlist-only";
    const level = isWatchlistOnly ? "info" : "warning";

    // 文件行数 warning
    const lineFloor = config.scc?.fileLines?.absoluteFloor ?? 300;
    const lineDelta = config.scc?.fileLines?.changedDelta ?? 100;
    const baseFile = baselineFiles.get(file.path);
    const baselineLines = baseFile?.lines ?? (hasBaselineFiles ? 0 : null);
    const lineDeltaValue = deltaFrom(file.lines, baselineLines);

    if (shouldWarn({
      isChanged: file.isChanged,
      isWatchlistOnly,
      value: file.lines,
      floor: lineFloor,
      delta: lineDeltaValue,
      deltaFloor: lineDelta
    })) {
      warnings.push({
        level,
        ruleId: "scc-file-lines",
        sourceTool: "scc",
        path: file.path,
        line: null,
        codeArea: file.codeArea,
        metric: "lines",
        value: file.lines,
        comparisonBasis: basisFor(file.isChanged, lineDeltaValue),
        baselineValue: baselineLines,
        deltaValue: lineDeltaValue,
        message: `File "${file.path}" has ${file.lines} lines (floor: ${lineFloor})`,
        suggestion: file.lines > lineFloor * 3
          ? "Consider splitting this file into smaller modules"
          : "Review if the file can be refactored"
      });
    }

    // 文件级复杂度 warning
    const ccFloor = config.scc?.fileComplexity?.absoluteFloor ?? 20;
    const ccDelta = config.scc?.fileComplexity?.changedDelta ?? 10;
    const baseComplexity = baseFile?.complexity?.value ?? (hasBaselineFiles ? 0 : null);
    const ccDeltaValue = deltaFrom(file.complexity.value, baseComplexity);
    if (shouldWarn({
      isChanged: file.isChanged,
      isWatchlistOnly,
      value: file.complexity.value,
      floor: ccFloor,
      delta: ccDeltaValue,
      deltaFloor: ccDelta
    })) {
      warnings.push({
        level,
        ruleId: "scc-file-complexity",
        sourceTool: "scc",
        path: file.path,
        line: null,
        codeArea: file.codeArea,
        metric: "complexity",
        value: file.complexity.value,
        comparisonBasis: basisFor(file.isChanged, ccDeltaValue),
        baselineValue: baseComplexity,
        deltaValue: ccDeltaValue,
        message: `File "${file.path}" has complexity ${file.complexity.value} (floor: ${ccFloor})`,
        suggestion: "Consider splitting complex logic into smaller functions"
      });
    }
  }

  // Function-level warnings (from Lizard data)
  for (const func of functions) {
    const areaConfig = config.codeAreas[func.codeArea];
    if (!areaConfig) continue;
    if (areaConfig.warningPolicy === "exclude-warnings") continue;

    const isWatchlistOnly = areaConfig.warningPolicy === "watchlist-only";
    const level = isWatchlistOnly ? "info" : "warning";

    // 圈复杂度 warning
    const ccFloor = config.lizard?.cyclomaticComplexity?.absoluteFloor ?? 10;
    const ccDelta = config.lizard?.cyclomaticComplexity?.changedDelta ?? 5;
    const baselineFunc = baselineFunctions.get(functionKey(func));
    const baselineCc = baselineFunc?.cyclomaticComplexity?.value ?? (hasBaselineFunctions ? 0 : null);
    const ccDeltaValue = deltaFrom(func.cyclomaticComplexity.value, baselineCc);
    if (shouldWarn({
      isChanged: func.isChanged,
      isWatchlistOnly,
      value: func.cyclomaticComplexity.value,
      floor: ccFloor,
      delta: ccDeltaValue,
      deltaFloor: ccDelta
    })) {
      warnings.push({
        level,
        ruleId: "lizard-cyclomatic-complexity",
        sourceTool: "lizard",
        path: func.file,
        line: func.startLine,
        codeArea: func.codeArea,
        metric: "cyclomatic-complexity",
        value: func.cyclomaticComplexity.value,
        comparisonBasis: basisFor(func.isChanged, ccDeltaValue),
        baselineValue: baselineCc,
        deltaValue: ccDeltaValue,
        message: `Function "${func.name}" in ${func.file}:${func.startLine} has cyclomatic complexity ${func.cyclomaticComplexity.value} (floor: ${ccFloor})`,
        suggestion: "Consider breaking this function into smaller, more focused functions"
      });
    }

    // 函数行数 warning
    const lineFloor = config.lizard?.functionLines?.absoluteFloor ?? 50;
    const lineDeltaCfg = config.lizard?.functionLines?.changedDelta ?? 20;
    const baselineFunctionLines = baselineFunc?.lines ?? (hasBaselineFunctions ? 0 : null);
    const functionLineDelta = deltaFrom(func.lines, baselineFunctionLines);
    if (shouldWarn({
      isChanged: func.isChanged,
      isWatchlistOnly,
      value: func.lines,
      floor: lineFloor,
      delta: functionLineDelta,
      deltaFloor: lineDeltaCfg
    })) {
      warnings.push({
        level,
        ruleId: "lizard-function-lines",
        sourceTool: "lizard",
        path: func.file,
        line: func.startLine,
        codeArea: func.codeArea,
        metric: "function-lines",
        value: func.lines,
        comparisonBasis: basisFor(func.isChanged, functionLineDelta),
        baselineValue: baselineFunctionLines,
        deltaValue: functionLineDelta,
        message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.lines} lines (floor: ${lineFloor})`,
        suggestion: "Consider extracting parts of this function into separate functions"
      });
    }

    // 参数数量 warning
    const paramFloor = config.lizard?.parameterCount?.absoluteFloor ?? 5;
    const paramDeltaCfg = config.lizard?.parameterCount?.changedDelta ?? 2;
    const baselineParameterCount = baselineFunc?.parameterCount ?? (hasBaselineFunctions ? 0 : null);
    const paramDeltaValue = deltaFrom(func.parameterCount, baselineParameterCount);
    if (shouldWarn({
      isChanged: func.isChanged,
      isWatchlistOnly,
      value: func.parameterCount,
      floor: paramFloor,
      delta: paramDeltaValue,
      deltaFloor: paramDeltaCfg
    })) {
      warnings.push({
        level,
        ruleId: "lizard-parameter-count",
        sourceTool: "lizard",
        path: func.file,
        line: func.startLine,
        codeArea: func.codeArea,
        metric: "parameter-count",
        value: func.parameterCount,
        comparisonBasis: basisFor(func.isChanged, paramDeltaValue),
        baselineValue: baselineParameterCount,
        deltaValue: paramDeltaValue,
        message: `Function "${func.name}" in ${func.file}:${func.startLine} has ${func.parameterCount} parameters (floor: ${paramFloor})`,
        suggestion: "Consider using a parameter object or splitting the function"
      });
    }
  }

  // Duplicate code warnings (from CPD)
  for (const dup of duplicates) {
    // 检查涉及的 code areas 的 warning policy
    const involvedAreas = dup.locations.map((l: ExternalValue) => l.codeArea).filter(Boolean);
    const uniqueAreas = [...new Set(involvedAreas)] as string[];

    // 如果所有涉及的 areas 都是 exclude-warnings，跳过
    if (uniqueAreas.length > 0 && uniqueAreas.every((a) => {
      const ac = config.codeAreas[a];
      return ac && ac.warningPolicy === "exclude-warnings";
    })) {
      continue;
    }

    // 如果只涉及 watchlist-only areas，使用 info level
    const isWatchlistOnly = uniqueAreas.length > 0 && uniqueAreas.every((a) => {
      const ac = config.codeAreas[a];
      return ac && ac.warningPolicy === "watchlist-only";
    });

    const level = isWatchlistOnly ? "info" : "warning";
    const primaryLocation = dup.locations[0];

    const baselineDuplicateCount = countMatchingBaselineDuplicates(
      dup,
      baselineDuplicateIndex,
      hasBaselineDuplicates
    );
    const duplicateDelta = baselineDuplicateCount === null ? null : 1 - baselineDuplicateCount;
    const duplicateDeltaFloor = config.pmdCpd?.duplicateFragments?.changedDelta ?? 0;

    if (shouldWarn({
      isChanged: dup.hitsChangedScope,
      isWatchlistOnly,
      value: dup.tokenCount,
      floor: 0,
      delta: duplicateDelta,
      deltaFloor: duplicateDeltaFloor
    })) {
      warnings.push({
        level,
        ruleId: "pmd-cpd-duplicate-code",
        sourceTool: "pmd-cpd",
        path: primaryLocation?.path || "ExternalValue",
        line: primaryLocation?.startLine ?? null,
        codeArea: uniqueAreas.join(",") || "ExternalValue",
        metric: "duplicate-tokens",
        value: dup.tokenCount,
        comparisonBasis: basisFor(dup.hitsChangedScope, duplicateDelta),
        baselineValue: baselineDuplicateCount,
        deltaValue: duplicateDelta,
        message: `Duplicate code fragment (${dup.tokenCount} tokens) across ${dup.locations.length} locations in areas [${uniqueAreas.join(", ")}]`,
        suggestion: `Consider extracting shared code into a common function or module. Locations: ${dup.locations.map((l: ExternalValue) => `${l.path}:${l.startLine}`).join(", ")}`
      });
    }
  }

  // 排序：error > warning > info，然后按 value 降序
  warnings.sort((a, b) => {
    const lvlOrder: Record<string, number> = { error: 0, warning: 1, info: 2 };
    const lvlDiff = lvlOrder[a.level] - lvlOrder[b.level];
    if (lvlDiff !== 0) return lvlDiff;
    return b.value - a.value;
  });

  return warnings;
}

function shouldWarn({ isChanged, isWatchlistOnly, value, floor, delta, deltaFloor }: ExternalValue) {
  if (value === null || value === undefined || value <= floor) {
    return false;
  }

  if (isWatchlistOnly) {
    return isChanged;
  }

  if (!isChanged) {
    return false;
  }

  if (delta === null || delta === undefined) {
    return true;
  }

  return delta > deltaFloor;
}

function basisFor(isChanged: ExternalValue, delta: ExternalValue) {
  if (isChanged && delta !== null && delta !== undefined) {
    return "delta";
  }
  return isChanged ? "changed-scope" : "absolute";
}

function deltaFrom(current: ExternalValue, baseline: ExternalValue) {
  if (current === null || current === undefined || baseline === null || baseline === undefined) {
    return null;
  }
  return current - baseline;
}

function buildFileBaselineMap(files: ExternalValue): Map<string, ExternalValue> {
  return new Map(files.map((file: ExternalValue) => [file.path, file]));
}

function buildFunctionBaselineMap(functions: ExternalValue): Map<string, ExternalValue> {
  return new Map(functions.map((func: ExternalValue) => [functionKey(func), func]));
}

function functionKey(func: ExternalValue) {
  return `${func.file}:${func.name}:${func.startLine}`;
}

function buildDuplicateBaselineIndex(duplicates: ExternalValue): Map<string, number> {
  const index = new Map<string, number>();
  for (const dup of duplicates) {
    const key = duplicateKey(dup);
    index.set(key, (index.get(key) || 0) + 1);
  }
  return index;
}

function countMatchingBaselineDuplicates(dup: ExternalValue, baselineIndex: ExternalValue, hasBaselineDuplicates: ExternalValue) {
  if (!hasBaselineDuplicates) {
    return null;
  }
  return baselineIndex.get(duplicateKey(dup)) || 0;
}

function duplicateKey(dup: ExternalValue) {
  return dup.locations
    .map((loc: ExternalValue) => `${loc.path}:${loc.startLine}`)
    .sort()
    .join("|");
}
