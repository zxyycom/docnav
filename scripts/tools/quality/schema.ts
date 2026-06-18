/**
 * Quality metrics JSON schema and validation for code-quality-observability.
 *
 * 定义统一 metrics JSON 结构，包含 schema version、扫描元数据、工具名称和版本、
 * 扫描范围、排除规则、code areas、current/baseline 扫描输入指纹、baseline metadata、
 * baseline status、comparison status、文件指标、函数指标、重复代码指标、聚合指标、
 * 趋势比较和 warning records。
 *
 * 来源：openspec/changes/implement-code-quality-observability/specs/code-quality-observability/spec.md
 */

// ── Constants ──────────────────────────────────────────────────────────

/** 当前 metrics schema version */
export const METRICS_SCHEMA_VERSION = "0.1.0";

/** 合法的 baseline status */
export const BASELINE_STATUSES = Object.freeze([
  "generated",
  "baseline-skipped",
  "history-unavailable",
  "no-baseline-commit",
  "baseline-materialization-failed",
  "baseline-scan-failed"
]);

/** 合法的 comparison status */
export const COMPARISON_STATUSES = Object.freeze([
  "compared",
  "input-unchanged",
  "baseline-unavailable"
]);

/** 合法的 warning levels */
export const WARNING_LEVELS = Object.freeze(["info", "warning", "error"]);

/** 合法的 code area warning policies */
export const WARNING_POLICIES = Object.freeze([
  "strict",
  "moderate",
  "relaxed",
  "watchlist-only",
  "exclude-warnings"
]);

// ── Type helpers (JSDoc, validated at runtime) ────────────────────────

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
    functionLines: QualityThreshold;
    parameterCount: QualityThreshold;
  };
  pmdCpd: {
    defaultMinimumTokens: number;
    duplicateFragments: { changedDelta: number };
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
    fileLines: QualityThreshold;
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
  warnings: WarningRecord[];
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

/**
 * @typedef {object} ScanMetadata
 * @property {string} schemaVersion - Metrics schema version
 * @property {string} timestamp - ISO 8601 scan timestamp
 * @property {string} repository - Repository root path
 * @property {string} commitSha - Current commit SHA
 * @property {string} [commitDate] - Current commit date (ISO 8601)
 * @property {ToolInfo[]} tools - Tool names and versions used
 * @property {{ include: string[], excludeDirs: string[], generatedFiles: string[] }} scope - Scan scope
 * @property {string} configVersion - Quality config version used
 */

/**
 * @typedef {object} ToolInfo
 * @property {string} name - Tool name (lizard, scc, pmd-cpd)
 * @property {string} version - Tool version string
 * @property {string} source - How the tool was resolved (e.g., "uv", "system", "path")
 */

/**
 * @typedef {object} CodeAreaFingerprint
 * @property {number} fileCount - Number of files in this code area
 * @property {string[]} fileList - Sorted list of normalized paths (may be truncated for large areas)
 * @property {string} fingerprint - Content fingerprint (e.g., git tree hash or blob hash)
 */

/**
 * @typedef {object} BaselineMetadata
 * @property {string} commitSha
 * @property {string} commitDate - ISO 8601
 * @property {string} selectionReason - How the baseline commit was selected
 * @property {string} configVersion - Quality config version used for baseline scan
 * @property {ToolInfo[]} toolMetadata - Tool versions used for baseline scan
 */

/**
 * @typedef {object} FileMetric
 * @property {string} path - Normalized file path relative to repo root
 * @property {string} language - Programming language
 * @property {string} codeArea - Code area classification
 * @property {number} lines - Total line count
 * @property {number} [codeLines] - Code line count (excl. comments/blanks)
 * @property {number} [commentLines] - Comment line count
 * @property {number} [blankLines] - Blank line count
 * @property {{ value: number|null, source: string }} complexity - File-level complexity. null if unavailable.
 * @property {boolean} isChanged - Whether the file was changed in current revision
 */

/**
 * @typedef {object} FunctionMetric
 * @property {string} name - Function or method name
 * @property {string} file - Normalized owning file path
 * @property {string} codeArea - Code area classification
 * @property {number} startLine - Starting line number (1-based)
 * @property {number} endLine - Ending line number (1-based)
 * @property {number} lines - Function line count (NLOC)
 * @property {number} parameterCount - Parameter count
 * @property {{ value: number|null, source: string }} cyclomaticComplexity - Cyclomatic complexity. null if unavailable.
 * @property {boolean} isChanged - Whether the owning file was changed in current revision
 */

/**
 * @typedef {object} DuplicateCodeFragment
 * @property {number} id - Fragment identifier (unique within this snapshot)
 * @property {number} tokenCount - CPD token count
 * @property {number} lineCount - Approximate line count
 * @property {{ path: string, startLine: number, endLine: number, codeArea: string }[]} locations
 * @property {string[]} codeAreas - Code areas involved in this duplication
 * @property {boolean} hitsChangedScope - Whether any involved file was changed
 */

/**
 * @typedef {object} AggregateMetrics
 * @property {LanguageAggregate[]} byLanguage - Aggregated by language
 * @property {CodeAreaAggregate[]} byCodeArea - Aggregated by code area
 * @property {{
 *   totalFiles: number,
 *   totalLines: number,
 *   totalCodeLines: number,
 *   totalFileComplexity?: number,
 *   totalFunctions: number,
 *   totalFunctionLines?: number,
 *   totalFunctionParameters?: number,
 *   totalFunctionCyclomaticComplexity?: number,
 *   totalDuplicateFragments?: number
 * }} overall
 */

/**
 * @typedef {object} LanguageAggregate
 * @property {string} language
 * @property {number} files
 * @property {number} lines
 * @property {number} codeLines
 * @property {number} commentLines
 * @property {number} blankLines
 * @property {number} [complexitySum]
 * @property {string} complexitySource
 */

/**
 * @typedef {object} CodeAreaAggregate
 * @property {string} codeArea
 * @property {number} files
 * @property {number} lines
 * @property {number} [codeLines]
 * @property {number} [fileComplexity]
 * @property {number} functions
 * @property {number} [functionLines]
 * @property {number} [parameterCount]
 * @property {number} [cyclomaticComplexity]
 * @property {number} [duplicateFragments]
 * @property {string} warningPolicy
 */

/**
 * @typedef {object} TrendDelta
 * @property {string} metric - Metric name
 * @property {number|null} current - Current value (null if not measurable)
 * @property {number|null} baseline - Baseline value (null if baseline unavailable)
 * @property {number|null} delta - Difference (current - baseline), null if either is null
 * @property {number|null} percentChange - Percentage change, null if baseline is 0 or null
 * @property {string} unit - Unit (count, percent, lines, etc.)
 */

/**
 * @typedef {object} WarningRecord
 * @property {string} level - info, warning, error
 * @property {string} ruleId - Stable rule identifier
 * @property {string} sourceTool - Tool that produced the finding
 * @property {string} path - Normalized file path
 * @property {number|null} line - Line number (null if file-level)
 * @property {string} codeArea - Code area classification
 * @property {string} metric - Metric name
 * @property {number} value - Current metric value
 * @property {string} comparisonBasis - "absolute", "delta", "changed-scope"
 * @property {number|null} baselineValue - Baseline value (null if unavailable)
 * @property {number|null} deltaValue - Delta value (null if unavailable)
 * @property {string} message - Human-readable message
 * @property {string} [suggestion] - Optional fix suggestion
 */

/**
 * @typedef {object} QualityMetrics
 * @property {ScanMetadata} metadata
 * @property {{ status: string, commitSha: string|null, commitDate: string|null, metadata: BaselineMetadata|null }} baseline
 * @property {string} comparisonStatus - compared, input-unchanged, baseline-unavailable
 * @property {Object<string, CodeAreaFingerprint>} currentFingerprints
 * @property {Object<string, CodeAreaFingerprint>} [baselineFingerprints]
 * @property {FileMetric[]} fileMetrics
 * @property {FunctionMetric[]} functionMetrics
 * @property {DuplicateCodeFragment[]} duplicateCode
 * @property {AggregateMetrics} aggregates
 * @property {TrendDelta[]} trends
 * @property {WarningRecord[]} warnings
 */

/**
 * @typedef {object} QualityConfig
 * @property {string} version
 * @property {string[]} include
 * @property {string[]} excludeDirs
 * @property {string[]} generatedFiles
 * @property {Object<string, CodeAreaDefinition>} codeAreas
 * @property {object} lizard
 * @property {object} scc
 * @property {object} pmdCpd
 * @property {object} report
 * @property {string} artifactDir
 * @property {object} tools
 */

/**
 * @typedef {object} CodeAreaDefinition
 * @property {string} description
 * @property {string[]} globs
 * @property {string[]} excludeGlobs
 * @property {string} warningPolicy
 */

/**
 * @typedef {object} ToolConfig
 * @property {string} command
 * @property {string[]} args
 */

import { isRecord, isUnknownArray } from "../types.ts";

// ── Validation ─────────────────────────────────────────────────────────

/**
 * 验证 metrics 对象是否符合 QualityMetrics schema。
 * 仅做结构检查，不深度验证数值语义。
 *
 * @param {QualityMetrics} metrics
 * @returns {{ valid: boolean, errors: string[] }}
 */
export function validateMetrics(metrics: unknown): MetricsValidationResult {
  const errors: string[] = [];

  if (!isRecord(metrics)) {
    return { valid: false, errors: ["metrics must be a non-null object"] };
  }

  // metadata
  const m = metrics.metadata;
  if (!isRecord(m)) {
    errors.push("metrics.metadata is required");
  } else {
    if (m.schemaVersion !== METRICS_SCHEMA_VERSION) {
      errors.push(
        `metadata.schemaVersion: expected "${METRICS_SCHEMA_VERSION}", got "${m.schemaVersion}"`
      );
    }
    if (!m.timestamp) errors.push("metadata.timestamp is required");
    if (!m.repository) errors.push("metadata.repository is required");
    if (!m.commitSha) errors.push("metadata.commitSha is required");
    if (!Array.isArray(m.tools)) errors.push("metadata.tools must be an array");
    if (!isRecord(m.scope)) errors.push("metadata.scope is required");
    if (!m.configVersion) errors.push("metadata.configVersion is required");
  }

  // baseline
  const baseline = metrics.baseline;
  if (!isRecord(baseline)) {
    errors.push("metrics.baseline is required");
  } else {
    const status = baseline.status;
    if (typeof status !== "string" || !BASELINE_STATUSES.includes(status)) {
      errors.push(
        `baseline.status: must be one of ${BASELINE_STATUSES.join(", ")}, got "${status}"`
      );
    }
  }

  // comparison status
  const comparisonStatus = metrics.comparisonStatus;
  if (typeof comparisonStatus !== "string" || !COMPARISON_STATUSES.includes(comparisonStatus)) {
    errors.push(
      `comparisonStatus: must be one of ${COMPARISON_STATUSES.join(", ")}, got "${comparisonStatus}"`
    );
  }

  // fingerprints
  if (!isRecord(metrics.currentFingerprints)) {
    errors.push("currentFingerprints is required");
  }

  // arrays
  if (!isUnknownArray(metrics.fileMetrics)) errors.push("fileMetrics must be an array");
  if (!isUnknownArray(metrics.functionMetrics)) errors.push("functionMetrics must be an array");
  if (!isUnknownArray(metrics.duplicateCode)) errors.push("duplicateCode must be an array");
  if (!isRecord(metrics.aggregates)) {
    errors.push("aggregates is required");
  }
  if (!isUnknownArray(metrics.trends)) errors.push("trends must be an array");
  if (!isUnknownArray(metrics.warnings)) errors.push("warnings must be an array");

  // validate warnings
  const warnings = metrics.warnings;
  if (isUnknownArray(warnings)) {
    for (let i = 0; i < warnings.length; i++) {
      const w = warnings[i];
      if (!isRecord(w)) {
        errors.push(`warnings[${i}] must be an object`);
        continue;
      }
      const level = w.level;
      if (typeof level !== "string" || !WARNING_LEVELS.includes(level)) {
        errors.push(`warnings[${i}].level: invalid level "${level}"`);
      }
      if (!w.ruleId) errors.push(`warnings[${i}].ruleId is required`);
      if (!w.message) errors.push(`warnings[${i}].message is required`);
    }
  }

  return { valid: errors.length === 0, errors };
}

/**
 * 生成空的 metrics 骨架。
 *
 * @param {object} params
 * @param {string} params.repository
 * @param {string} params.commitSha
 * @param {string} params.configVersion
 * @param {ToolInfo[]} params.tools
 * @param {{ include: string[], excludeDirs: string[], generatedFiles: string[] }} params.scope
 * @returns {QualityMetrics}
 */
export function createEmptyMetrics({
  repository,
  commitSha,
  configVersion,
  tools,
  scope
}: {
  configVersion: string;
  commitSha: string;
  repository: string;
  scope: ScanMetadata["scope"];
  tools: ToolInfo[];
}): QualityMetrics {
  return {
    metadata: {
      schemaVersion: METRICS_SCHEMA_VERSION,
      timestamp: new Date().toISOString(),
      repository,
      commitSha,
      tools,
      scope,
      configVersion
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
    warnings: []
  };
}
