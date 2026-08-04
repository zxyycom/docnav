export {
  BASELINE_STATUSES,
  COMPARISON_STATUSES,
  DUPLICATE_CODE_MEASUREMENT_STATUSES,
  METRICS_SCHEMA_VERSION,
  WARNING_LEVELS,
  WARNING_POLICIES
} from "./schema/types.ts";
export type {
  AcceptedWarningConfig,
  AggregateMetrics,
  BaselineMetadata,
  BaselineSnapshot,
  BaselineStatus,
  CodeAreaAggregate,
  CodeAreaDefinition,
  CodeAreaFileMap,
  CodeAreaFingerprint,
  CodeAreaWarningPolicy,
  ComparisonStatus,
  DuplicateCodeMeasurementStatus,
  DuplicateCodeFragment,
  DuplicateCodeLocation,
  FatalIssue,
  FileMetric,
  FunctionMetric,
  LanguageAggregate,
  MetricValue,
  MetricsValidationResult,
  QualityConfig,
  QualityMetrics,
  QualityThreshold,
  ScanMetadata,
  ToolAvailability,
  ToolConfig,
  ToolInfo,
  TrendDelta,
  WarningChannels,
  WarningLevel,
  WarningRecord
} from "./schema/types.ts";
export { createEmptyMetrics } from "./schema/empty-metrics.ts";
export { validateMetrics } from "./schema/validation.ts";
