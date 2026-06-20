/**
 * Quality metrics JSON schema and validation for code-quality-observability.
 *
 * 定义统一 metrics JSON 结构，包含 schema version、扫描元数据、工具名称和版本、
 * 扫描范围、排除规则、code areas、current/baseline 扫描输入指纹、baseline metadata、
 * baseline status、comparison status、文件指标、函数指标、重复代码指标、聚合指标、
 * 趋势比较和 warning records。
 */

import { isRecord, isUnknownArray } from "../../type-guards.ts";

// ── Constants ──────────────────────────────────────────────────────────

export const METRICS_SCHEMA_VERSION = "0.2.1";

export const BASELINE_STATUSES = Object.freeze([
  "generated",
  "baseline-skipped",
  "history-unavailable",
  "no-baseline-commit",
  "baseline-materialization-failed",
  "baseline-scan-failed"
]);

export const COMPARISON_STATUSES = Object.freeze([
  "compared",
  "input-unchanged",
  "baseline-unavailable"
]);

export const WARNING_LEVELS = Object.freeze(["info", "warning", "error"]);

export const WARNING_POLICIES = Object.freeze([
  "strict",
  "moderate",
  "relaxed",
  "watchlist-only",
  "exclude-warnings"
]);

// ── Types ─────────────────────────────────────────────────────────────

export type BaselineStatus = typeof BASELINE_STATUSES[number];
export type CodeAreaWarningPolicy = typeof WARNING_POLICIES[number];
export type ComparisonStatus = typeof COMPARISON_STATUSES[number];
export type WarningLevel = typeof WARNING_LEVELS[number];

export interface ToolInfo {
  name: string;
  source: string;
  version: string;
}

export interface ToolAvailability {
  available: boolean;
  error?: string | null;
  name: string;
  reason?: string | null;
  source: string;
  version: string | null;
}

export interface ToolConfig {
  args: string[];
  command: string;
}

export interface CodeAreaDefinition {
  description: string;
  excludeGlobs: string[];
  globs: string[];
  warningPolicy: CodeAreaWarningPolicy;
}

export interface QualityConfig {
  artifactDir: string;
  codeAreas: Record<string, CodeAreaDefinition>;
  excludeDirs: string[];
  generatedFiles: string[];
  include: string[];
  lizard: {
    cyclomaticComplexity: QualityThreshold;
    functionCodeLines: QualityThreshold;
    parameterCount: QualityThreshold;
  };
  pmdCpd: {
    defaultMinimumTokens: number;
    duplicateFragments: { changedDelta: number };
    maxParallelTasks: number;
    minimumTokens: Record<string, number>;
  };
  report: {
    showWatchlist: boolean;
    timeZone: string;
    topN: number;
    watchlistMax: number;
  };
  scc: {
    fileComplexity: QualityThreshold;
    fileCodeLines: QualityThreshold;
  };
  tools: {
    lizard: ToolConfig;
    pmdCpd: ToolConfig;
    scc: ToolConfig;
  };
  version: string;
}

export interface QualityThreshold {
  absoluteFloor: number;
  changedDelta: number;
}

export interface ScanMetadata {
  commitDate?: string;
  commitSha: string;
  commitTitle: string | null;
  configVersion: string;
  repository: string;
  schemaVersion: string;
  scope: {
    excludeDirs: string[];
    generatedFiles: string[];
    include: string[];
  };
  timestamp: string;
  tools: ToolInfo[];
}

export interface CodeAreaFingerprint {
  fileCount: number;
  fileList: string[];
  fingerprint: string;
}

export type CodeAreaFileMap = Map<string, string[]>;

export interface BaselineMetadata {
  commitDate: string | null;
  commitSha: string;
  commitTitle: string | null;
  configVersion: string;
  selectionReason: string;
  toolMetadata: ToolInfo[];
}

export interface ComplexityValue {
  source: string;
  value: number | null;
}

export interface FileMetric {
  blankLines?: number;
  codeArea: string;
  codeLines?: number;
  commentLines?: number;
  complexity: ComplexityValue;
  isChanged: boolean;
  language: string;
  lines: number;
  path: string;
}

export interface FunctionMetric {
  codeArea: string;
  cyclomaticComplexity: ComplexityValue;
  endLine: number;
  file: string;
  isChanged: boolean;
  lines: number;
  name: string;
  parameterCount: number;
  startLine: number;
}

export interface DuplicateCodeLocation {
  codeArea: string;
  endLine: number;
  path: string;
  startLine: number;
}

export interface DuplicateCodeFragment {
  codeAreas: string[];
  hitsChangedScope: boolean;
  id: number;
  lineCount: number;
  locations: DuplicateCodeLocation[];
  tokenCount: number;
}

export interface LanguageAggregate {
  blankLines: number;
  codeLines: number;
  comments?: number;
  commentLines: number;
  complexitySource: string;
  complexitySum?: number;
  files: number;
  language: string;
  lines: number;
}

export interface CodeAreaAggregate {
  codeArea: string;
  codeLines?: number;
  cyclomaticComplexity?: number;
  duplicateFragments?: number;
  fileComplexity?: number;
  files: number;
  functionLines?: number;
  functions: number;
  lines: number;
  parameterCount?: number;
  warningPolicy: CodeAreaWarningPolicy | string;
}

export interface AggregateMetrics {
  byCodeArea: CodeAreaAggregate[];
  byLanguage: LanguageAggregate[];
  overall: {
    totalCodeLines: number;
    totalDuplicateFragments?: number;
    totalFileComplexity?: number;
    totalFiles: number;
    totalFunctionCyclomaticComplexity?: number;
    totalFunctionLines?: number;
    totalFunctionParameters?: number;
    totalFunctions: number;
    totalLines: number;
  };
}

export interface TrendDelta {
  baseline: number | null;
  current: number | null;
  delta: number | null;
  metric: string;
  percentChange: number | null;
  unit: string;
}

export interface WarningRecord {
  baselineValue: number | null;
  codeArea: string;
  comparisonBasis: string;
  deltaValue: number | null;
  isChanged: boolean;
  level: WarningLevel | string;
  line: number | null;
  message: string;
  metric: string;
  path: string;
  ruleId: string;
  sourceTool: string;
  suggestion?: string;
  value: number;
}

export interface WarningChannels {
  all: WarningRecord[];
  changed: WarningRecord[];
  regressions: WarningRecord[];
}

export interface QualityMetrics {
  aggregates: AggregateMetrics;
  baseline: {
    commitDate: string | null;
    commitSha: string | null;
    metadata: BaselineMetadata | null;
    status: BaselineStatus | string;
  };
  baselineFingerprints?: Record<string, CodeAreaFingerprint>;
  comparisonStatus: ComparisonStatus | string;
  currentFingerprints: Record<string, CodeAreaFingerprint>;
  duplicateCode: DuplicateCodeFragment[];
  fileMetrics: FileMetric[];
  functionMetrics: FunctionMetric[];
  metadata: ScanMetadata;
  trends: TrendDelta[];
  warnings: WarningChannels;
}

export interface BaselineSnapshot {
  aggregates: AggregateMetrics;
  duplicateCode: DuplicateCodeFragment[];
  fileMetrics: FileMetric[];
  fingerprints: Record<string, CodeAreaFingerprint>;
  functionMetrics: FunctionMetric[];
}

export interface FatalIssue {
  error: string;
  phase: string;
  tool: string;
}

export interface MetricsValidationResult {
  errors: string[];
  valid: boolean;
}

// ── Validation ─────────────────────────────────────────────────────────

/**
 * 验证 metrics 对象是否符合 QualityMetrics schema。
 * 仅做结构检查，不深度验证数值语义。
 */
export function validateMetrics(metrics: unknown): MetricsValidationResult {
  const errors: string[] = [];

  if (!isRecord(metrics)) {
    return { valid: false, errors: ["metrics must be a non-null object"] };
  }

  validateMetadata(metrics.metadata, errors);
  validateBaseline(metrics.baseline, errors);
  validateStatusField(
    metrics.comparisonStatus,
    COMPARISON_STATUSES,
    "comparisonStatus",
    errors
  );
  validateRequiredObjects(metrics, errors);
  validateMetricArrays(metrics, errors);
  validateWarningChannels(metrics.warnings, errors);

  return { valid: errors.length === 0, errors };
}

function validateMetadata(metadata: unknown, errors: string[]): void {
  if (!isRecord(metadata)) {
    errors.push("metrics.metadata is required");
    return;
  }

  validateExactValue(
    metadata.schemaVersion,
    METRICS_SCHEMA_VERSION,
    "metadata.schemaVersion",
    errors
  );
  requireTruthyField(metadata.timestamp, "metadata.timestamp", errors);
  requireTruthyField(metadata.repository, "metadata.repository", errors);
  requireTruthyField(metadata.commitSha, "metadata.commitSha", errors);
  requireArrayField(metadata.tools, "metadata.tools", errors);
  requireRecordField(metadata.scope, "metadata.scope", errors);
  requireTruthyField(metadata.configVersion, "metadata.configVersion", errors);
}

function validateBaseline(baseline: unknown, errors: string[]): void {
  if (!isRecord(baseline)) {
    errors.push("metrics.baseline is required");
    return;
  }

  validateStatusField(baseline.status, BASELINE_STATUSES, "baseline.status", errors);
}

function validateRequiredObjects(metrics: Record<string, unknown>, errors: string[]): void {
  requireRecordField(metrics.currentFingerprints, "currentFingerprints", errors);
  requireRecordField(metrics.aggregates, "aggregates", errors);
}

function validateMetricArrays(metrics: Record<string, unknown>, errors: string[]): void {
  requireUnknownArrayField(metrics.fileMetrics, "fileMetrics", errors);
  requireUnknownArrayField(metrics.functionMetrics, "functionMetrics", errors);
  requireUnknownArrayField(metrics.duplicateCode, "duplicateCode", errors);
  requireUnknownArrayField(metrics.trends, "trends", errors);
}

function validateWarningChannels(warnings: unknown, errors: string[]): void {
  if (!isRecord(warnings)) {
    errors.push("warnings must be an object with all, changed, and regressions arrays");
    return;
  }

  for (const channel of ["all", "changed", "regressions"] as const) {
    const channelWarnings = warnings[channel];
    if (!isUnknownArray(channelWarnings)) {
      errors.push(`warnings.${channel} must be an array`);
      continue;
    }
    validateWarningRecords(channelWarnings, `warnings.${channel}`, errors);
  }
}

function validateWarningRecords(warnings: unknown[], prefix: string, errors: string[]): void {
  for (let i = 0; i < warnings.length; i++) {
    validateWarningRecord(warnings[i], `${prefix}[${i}]`, errors);
  }
}

function validateWarningRecord(warning: unknown, prefix: string, errors: string[]): void {
  if (!isRecord(warning)) {
    errors.push(`${prefix} must be an object`);
    return;
  }

  validateWarningLevel(warning.level, `${prefix}.level`, errors);
  requireTruthyField(warning.ruleId, `${prefix}.ruleId`, errors);
  requireTruthyField(warning.message, `${prefix}.message`, errors);
}

function validateStatusField(
  value: unknown,
  allowedValues: readonly string[],
  fieldName: string,
  errors: string[]
): void {
  if (typeof value === "string" && allowedValues.includes(value)) return;
  errors.push(`${fieldName}: must be one of ${allowedValues.join(", ")}, got "${value}"`);
}

function validateWarningLevel(value: unknown, fieldName: string, errors: string[]): void {
  if (typeof value === "string" && WARNING_LEVELS.includes(value)) return;
  errors.push(`${fieldName}: invalid level "${value}"`);
}

function validateExactValue(
  value: unknown,
  expected: string,
  fieldName: string,
  errors: string[]
): void {
  if (value === expected) return;
  errors.push(`${fieldName}: expected "${expected}", got "${value}"`);
}

function requireTruthyField(value: unknown, fieldName: string, errors: string[]): void {
  if (!value) errors.push(`${fieldName} is required`);
}

function requireArrayField(value: unknown, fieldName: string, errors: string[]): void {
  if (!Array.isArray(value)) errors.push(`${fieldName} must be an array`);
}

function requireRecordField(value: unknown, fieldName: string, errors: string[]): void {
  if (!isRecord(value)) errors.push(`${fieldName} is required`);
}

function requireUnknownArrayField(value: unknown, fieldName: string, errors: string[]): void {
  if (!isUnknownArray(value)) errors.push(`${fieldName} must be an array`);
}

export function createEmptyMetrics(options: {
  configVersion: string;
  commitSha: string;
  commitTitle?: string | null;
  repository: string;
  scope: ScanMetadata["scope"];
  tools: ToolInfo[];
}): QualityMetrics {
  return {
    metadata: {
      schemaVersion: METRICS_SCHEMA_VERSION,
      timestamp: new Date().toISOString(),
      repository: options.repository,
      commitSha: options.commitSha,
      commitTitle: options.commitTitle ?? null,
      tools: options.tools,
      scope: options.scope,
      configVersion: options.configVersion
    },
    baseline: {
      status: "history-unavailable",
      commitSha: null,
      commitDate: null,
      metadata: null
    },
    comparisonStatus: "baseline-unavailable",
    currentFingerprints: {},
    fileMetrics: [],
    functionMetrics: [],
    duplicateCode: [],
    aggregates: {
      byLanguage: [],
      byCodeArea: [],
      overall: { totalFiles: 0, totalLines: 0, totalCodeLines: 0, totalFunctions: 0 }
    },
    trends: [],
    warnings: {
      all: [],
      changed: [],
      regressions: []
    }
  };
}
