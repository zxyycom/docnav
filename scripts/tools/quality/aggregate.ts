/**
 * Code quality aggregate builders.
 *
 * Keeps summary math separate from CLI orchestration and tool wrappers.
 */

import type {
  AggregateMetrics,
  CodeAreaAggregate,
  DuplicateCodeFragment,
  FileMetric,
  FunctionMetric,
  LanguageAggregate,
  QualityConfig
} from "./schema.ts";

export function buildAggregates({
  fileMetrics,
  functionMetrics,
  duplicateCode,
  byLanguage,
  config
}: {
  byLanguage: LanguageAggregate[];
  config: QualityConfig;
  duplicateCode: DuplicateCodeFragment[];
  fileMetrics: FileMetric[];
  functionMetrics: FunctionMetric[];
}): AggregateMetrics {
  const duplicateByArea = new Map<string, number>();
  for (const dup of duplicateCode) {
    for (const area of dup.codeAreas || []) {
      duplicateByArea.set(area, (duplicateByArea.get(area) || 0) + 1);
    }
  }

  const functionByArea = new Map<string, {
    cyclomaticComplexity: number;
    functionLines: number;
    functions: number;
    parameterCount: number;
  }>();
  for (const func of functionMetrics) {
    const existing = functionByArea.get(func.codeArea) || {
      functions: 0,
      functionLines: 0,
      parameterCount: 0,
      cyclomaticComplexity: 0
    };
    existing.functions++;
    existing.functionLines += func.lines || 0;
    existing.parameterCount += func.parameterCount || 0;
    existing.cyclomaticComplexity += func.cyclomaticComplexity?.value ?? 0;
    functionByArea.set(func.codeArea, existing);
  }

  const areaAggMap = new Map<string, CodeAreaAggregate>();
  for (const file of fileMetrics) {
    const existing = areaAggMap.get(file.codeArea) || createCodeAreaAggregate(file.codeArea, config);
    existing.files++;
    existing.lines += file.lines || 0;
    existing.codeLines = (existing.codeLines ?? 0) + (file.codeLines || 0);
    existing.fileComplexity = (existing.fileComplexity ?? 0) + (file.complexity?.value ?? 0);
    areaAggMap.set(file.codeArea, existing);
  }

  for (const [area, funcAgg] of functionByArea.entries()) {
    const existing = areaAggMap.get(area) || createCodeAreaAggregate(area, config);
    existing.functions = funcAgg.functions;
    existing.functionLines = funcAgg.functionLines;
    existing.parameterCount = funcAgg.parameterCount;
    existing.cyclomaticComplexity = funcAgg.cyclomaticComplexity;
    areaAggMap.set(area, existing);
  }

  for (const [area, count] of duplicateByArea.entries()) {
    const existing = areaAggMap.get(area) || createCodeAreaAggregate(area, config);
    existing.duplicateFragments = count;
    areaAggMap.set(area, existing);
  }

  const overall = {
    totalFiles: fileMetrics.length,
    totalLines: sum(fileMetrics, (file) => file.lines || 0),
    totalCodeLines: sum(fileMetrics, (file) => file.codeLines || 0),
    totalFileComplexity: sum(fileMetrics, (file) => file.complexity?.value ?? 0),
    totalFunctions: functionMetrics.length,
    totalFunctionLines: sum(functionMetrics, (func) => func.lines || 0),
    totalFunctionParameters: sum(functionMetrics, (func) => func.parameterCount || 0),
    totalFunctionCyclomaticComplexity: sum(
      functionMetrics,
      (func) => func.cyclomaticComplexity?.value ?? 0
    ),
    totalDuplicateFragments: duplicateCode.length
  };

  return {
    byLanguage,
    byCodeArea: Array.from(areaAggMap.values()).sort((a, b) => b.lines - a.lines),
    overall
  };
}

function createCodeAreaAggregate(codeArea: string, config: QualityConfig): CodeAreaAggregate {
  return {
    codeArea,
    files: 0,
    lines: 0,
    codeLines: 0,
    fileComplexity: 0,
    functions: 0,
    functionLines: 0,
    parameterCount: 0,
    cyclomaticComplexity: 0,
    duplicateFragments: 0,
    warningPolicy: config.codeAreas[codeArea]?.warningPolicy || "moderate"
  };
}

function sum<T>(items: readonly T[], selector: (item: T) => number): number {
  return items.reduce((total, item) => total + selector(item), 0);
}
