import type {
  NativeTestEntry,
  NativeTestInventory,
  SourceRange
} from "./model.ts";
import { isSafeRelativePosixPath } from "./relative-path.ts";

const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const FINGERPRINT_PATTERN = /^sha256:[0-9a-f]{64}$/;
const INVENTORY_KEYS = [
  "schemaVersion",
  "profile",
  "sourceRevision",
  "entries"
] as const;
const PROFILE_KEYS = ["id", "version"] as const;
const ENTRY_KEYS = [
  "entryKey",
  "runner",
  "target",
  "selector",
  "sourcePath",
  "sourceRange",
  "sourceFingerprint"
] as const;
const SOURCE_RANGE_KEYS = [
  "startLine",
  "startColumn",
  "endLine",
  "endColumn"
] as const;

type InventoryRoot = Record<string, unknown> & {
  schemaVersion: unknown;
  profile: unknown;
  sourceRevision: unknown;
  entries: unknown;
};

export function parseNativeTestInventory(value: unknown): NativeTestInventory {
  if (!isInventoryRoot(value) || value.schemaVersion !== 1) {
    throw new Error("native test inventory has an invalid root shape");
  }
  if (!isInventoryProfile(value.profile)) {
    throw new Error("native test inventory profile is invalid");
  }
  if (!isFingerprint(value.sourceRevision)) {
    throw new Error("native test inventory sourceRevision is invalid");
  }
  if (!Array.isArray(value.entries)) {
    throw new Error("native test inventory entries must be an array");
  }
  const invalidEntryIndex = value.entries.findIndex(
    (entry) => !isNativeTestEntry(entry)
  );
  if (invalidEntryIndex >= 0) {
    throw new Error(
      `native test inventory entries[${invalidEntryIndex}] is invalid`
    );
  }
  return value as NativeTestInventory;
}

export function isNativeTestEntry(value: unknown): value is NativeTestEntry {
  if (!isExactRecord(value, ENTRY_KEYS)) {
    return false;
  }
  return (
    isNonEmptyString(value.entryKey) &&
    isSlug(value.runner) &&
    isNonEmptyString(value.target) &&
    isNonEmptyString(value.selector) &&
    isSafeRelativePosixPath(value.sourcePath) &&
    isSourceRange(value.sourceRange) &&
    isFingerprint(value.sourceFingerprint)
  );
}

function isInventoryRoot(value: unknown): value is InventoryRoot {
  return isExactRecord(value, INVENTORY_KEYS);
}

function isInventoryProfile(
  value: unknown
): value is NativeTestInventory["profile"] {
  return (
    isExactRecord(value, PROFILE_KEYS) &&
    isSlug(value.id) &&
    isPositiveInteger(value.version)
  );
}

function isSourceRange(value: unknown): value is SourceRange {
  return (
    isExactRecord(value, SOURCE_RANGE_KEYS) &&
    isPositiveInteger(value.startLine) &&
    isPositiveInteger(value.startColumn) &&
    isPositiveInteger(value.endLine) &&
    isPositiveInteger(value.endColumn)
  );
}

function isExactRecord(
  value: unknown,
  keys: readonly string[]
): value is Record<string, unknown> {
  return (
    isRecord(value) &&
    JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...keys].sort())
  );
}

function isSlug(value: unknown): value is string {
  return typeof value === "string" && SLUG_PATTERN.test(value);
}

function isFingerprint(value: unknown): value is string {
  return typeof value === "string" && FINGERPRINT_PATTERN.test(value);
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isPositiveInteger(value: unknown): value is number {
  return Number.isInteger(value) && Number(value) >= 1;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
