import {
  isNonArrayRecord,
  isUnknownArray
} from "../../../../foundation/src/index.ts";
import {
  BASELINE_STATUSES,
  COMPARISON_STATUSES,
  DUPLICATE_CODE_MEASUREMENT_STATUSES,
  METRICS_SCHEMA_VERSION
} from "./types.ts";
import type { MetricsValidationResult } from "./types.ts";
import { validateWarningChannels } from "./warning-validation.ts";

/**
 * 验证 metrics 对象是否符合 QualityMetrics schema。
 * 仅做结构检查，不深度验证数值语义。
 */
export function validateMetrics(metrics: unknown): MetricsValidationResult {
  const errors: string[] = [];

  if (!isNonArrayRecord(metrics)) {
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
  validateDuplicateCodeMeasurement(metrics.duplicateCodeMeasurement, errors);
  validateRequiredObjects(metrics, errors);
  validateMetricArrays(metrics, errors);
  validateWarningChannels(metrics.warnings, errors);

  return { valid: errors.length === 0, errors };
}

function validateDuplicateCodeMeasurement(measurement: unknown, errors: string[]): void {
  if (measurement === undefined) {
    errors.push("duplicateCodeMeasurement is required");
    return;
  }
  if (!isNonArrayRecord(measurement)) {
    errors.push("duplicateCodeMeasurement must be an object");
    return;
  }
  validateStatusField(
    measurement.status,
    DUPLICATE_CODE_MEASUREMENT_STATUSES,
    "duplicateCodeMeasurement.status",
    errors
  );
}

function validateMetadata(metadata: unknown, errors: string[]): void {
  if (!isNonArrayRecord(metadata)) {
    errors.push("metrics.metadata is required");
    return;
  }

  validateExactValue(
    metadata.schemaVersion,
    METRICS_SCHEMA_VERSION,
    "metadata.schemaVersion",
    errors
  );
  requireNonEmptyStringField(metadata.timestamp, "metadata.timestamp", errors);
  requireNonEmptyStringField(metadata.repository, "metadata.repository", errors);
  requireNonEmptyStringField(metadata.commitSha, "metadata.commitSha", errors);
  requireArrayField(metadata.tools, "metadata.tools", errors);
  requireRecordField(metadata.scope, "metadata.scope", errors);
  requireNonEmptyStringField(metadata.configVersion, "metadata.configVersion", errors);
}

function validateBaseline(baseline: unknown, errors: string[]): void {
  if (!isNonArrayRecord(baseline)) {
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

function validateStatusField(
  value: unknown,
  allowedValues: readonly string[],
  fieldName: string,
  errors: string[]
): void {
  if (typeof value === "string" && allowedValues.includes(value)) return;
  errors.push(
    `${fieldName}: must be one of ${allowedValues.join(", ")}, got "${String(value)}"`
  );
}

function validateExactValue(
  value: unknown,
  expected: string,
  fieldName: string,
  errors: string[]
): void {
  if (value === expected) return;
  errors.push(`${fieldName}: expected "${expected}", got "${String(value)}"`);
}

function requireNonEmptyStringField(
  value: unknown,
  fieldName: string,
  errors: string[]
): void {
  if (typeof value !== "string" || value.length === 0) {
    errors.push(`${fieldName} must be a non-empty string`);
  }
}

function requireArrayField(value: unknown, fieldName: string, errors: string[]): void {
  if (!Array.isArray(value)) errors.push(`${fieldName} must be an array`);
}

function requireRecordField(value: unknown, fieldName: string, errors: string[]): void {
  if (!isNonArrayRecord(value)) errors.push(`${fieldName} must be an object`);
}

function requireUnknownArrayField(value: unknown, fieldName: string, errors: string[]): void {
  if (!isUnknownArray(value)) errors.push(`${fieldName} must be an array`);
}
