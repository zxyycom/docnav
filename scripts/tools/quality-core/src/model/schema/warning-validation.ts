import {
  isNonArrayRecord,
  isUnknownArray
} from "../../../../foundation/src/index.ts";
import { WARNING_LEVELS } from "./types.ts";

export function validateWarningChannels(warnings: unknown, errors: string[]): void {
  if (!isNonArrayRecord(warnings)) {
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
  if (!isNonArrayRecord(warning)) {
    errors.push(`${prefix} must be an object`);
    return;
  }

  validateWarningLevel(warning.level, `${prefix}.level`, errors);
  requireNonEmptyStringField(warning.ruleId, `${prefix}.ruleId`, errors);
  requireNonEmptyStringField(warning.message, `${prefix}.message`, errors);
  if (warning.acceptedReason !== undefined && typeof warning.acceptedReason !== "string") {
    errors.push(`${prefix}.acceptedReason must be a string when present`);
  }
}

function validateWarningLevel(value: unknown, fieldName: string, errors: string[]): void {
  if (typeof value === "string" && WARNING_LEVELS.includes(value)) return;
  errors.push(`${fieldName}: invalid level "${String(value)}"`);
}

function requireNonEmptyStringField(value: unknown, fieldName: string, errors: string[]): void {
  if (typeof value !== "string" || value.length === 0) {
    errors.push(`${fieldName} must be a non-empty string`);
  }
}
