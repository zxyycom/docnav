/**
 * Code-area cache for normalized PMD CPD duplicate-code results.
 */

import { createHash } from "node:crypto";
import { join } from "node:path";

import { readJsonFile, writeJsonFile } from "../../fs.ts";
import { isNonArrayRecord } from "../../type-guards.ts";
import type {
  CodeAreaFingerprint,
  DuplicateCodeFragment,
  DuplicateCodeLocation
} from "../model/schema.ts";

export const SCAN_CACHE_VERSION = "quality-scan-cache-v1";

export type ScanKind = "baseline" | "current";

export type CpdCacheIdentity = {
  codeArea: string;
  commitSha: string;
  configVersion: string;
  inputFingerprint: CodeAreaFingerprint;
  normalizedToolArgs: readonly string[];
  scanKind: ScanKind;
  toolName: "pmd-cpd";
  toolVersion: string;
};

export type CpdCacheHit = {
  cacheKey: string;
  cachePath: string;
  hit: true;
  metrics: DuplicateCodeFragment[];
};

export type CpdCacheMiss = {
  cacheKey: string;
  cachePath: string;
  hit: false;
  reason: string;
};

type ScanCachePayload = {
  cacheKey: string;
  codeArea: string;
  commitSha: string;
  configVersion: string;
  createdAt: string;
  inputFingerprint: CodeAreaFingerprint;
  metrics: unknown;
  normalizedToolArgs: string[];
  scanCacheVersion: string;
  scanKind: ScanKind;
  toolName: "pmd-cpd";
  toolVersion: string;
};

export function buildScanCacheKey(identity: CpdCacheIdentity): string {
  const keyInput = {
    scan_cache_version: SCAN_CACHE_VERSION,
    scan_kind: identity.scanKind,
    tool_name: identity.toolName,
    tool_version: identity.toolVersion,
    normalized_tool_args: [...identity.normalizedToolArgs],
    config_version: identity.configVersion,
    code_area: identity.codeArea,
    commit_sha: identity.commitSha,
    input_fingerprint: identity.inputFingerprint
  };

  return createHash("sha256").update(stableStringify(keyInput)).digest("hex");
}

export function getScanCachePath(rootDir: string, cacheKey: string): string {
  return join(rootDir, ".log", "docnav-quality-cache", SCAN_CACHE_VERSION, `${cacheKey}.json`);
}

export function loadScanCacheEntry({
  rootDir,
  identity
}: {
  identity: CpdCacheIdentity;
  rootDir: string;
}): CpdCacheHit | CpdCacheMiss {
  const cacheKey = buildScanCacheKey(identity);
  const cachePath = getScanCachePath(rootDir, cacheKey);

  let payload: unknown;
  try {
    payload = readJsonFile(cachePath);
  } catch {
    return { hit: false, reason: "cache-miss", cacheKey, cachePath };
  }

  if (!isMatchingPayload(payload, identity, cacheKey)) {
    return { hit: false, reason: "cache-payload-mismatch", cacheKey, cachePath };
  }

  if (!isMetricArray(payload.metrics)) {
    return { hit: false, reason: "cache-payload-invalid", cacheKey, cachePath };
  }

  return { hit: true, metrics: payload.metrics, cacheKey, cachePath };
}

export function writeScanCacheEntry({
  rootDir,
  identity,
  metrics
}: {
  identity: CpdCacheIdentity;
  metrics: DuplicateCodeFragment[];
  rootDir: string;
}): { cacheKey: string; cachePath: string } {
  const cacheKey = buildScanCacheKey(identity);
  const cachePath = getScanCachePath(rootDir, cacheKey);
  const payload: ScanCachePayload = {
    scanCacheVersion: SCAN_CACHE_VERSION,
    cacheKey,
    scanKind: identity.scanKind,
    toolName: identity.toolName,
    toolVersion: identity.toolVersion,
    normalizedToolArgs: [...identity.normalizedToolArgs],
    configVersion: identity.configVersion,
    codeArea: identity.codeArea,
    commitSha: identity.commitSha,
    inputFingerprint: identity.inputFingerprint,
    metrics: stripDuplicateChangedScope(metrics),
    createdAt: new Date().toISOString()
  };

  writeJsonFile(cachePath, payload);
  return { cacheKey, cachePath };
}

function stripDuplicateChangedScope(metrics: DuplicateCodeFragment[]): DuplicateCodeFragment[] {
  return metrics.map((fragment) => ({
    ...fragment,
    codeAreas: [...fragment.codeAreas],
    hitsChangedScope: false,
    locations: fragment.locations.map((location) => ({ ...location }))
  }));
}

function isMatchingPayload(
  payload: unknown,
  identity: CpdCacheIdentity,
  cacheKey: string
): payload is ScanCachePayload {
  if (!isNonArrayRecord(payload)) return false;

  return cacheIdentityFieldsMatch(payload, identity, cacheKey) &&
    cacheStructuredFieldsMatch(payload, identity);
}

function cacheIdentityFieldsMatch(
  payload: Record<string, unknown>,
  identity: CpdCacheIdentity,
  cacheKey: string
): boolean {
  return payload.scanCacheVersion === SCAN_CACHE_VERSION &&
    payload.cacheKey === cacheKey &&
    payload.scanKind === identity.scanKind &&
    payload.toolName === identity.toolName &&
    payload.toolVersion === identity.toolVersion &&
    payload.configVersion === identity.configVersion &&
    payload.codeArea === identity.codeArea &&
    payload.commitSha === identity.commitSha;
}

function cacheStructuredFieldsMatch(payload: Record<string, unknown>, identity: CpdCacheIdentity): boolean {
  return stableStringify(payload.normalizedToolArgs) === stableStringify([...identity.normalizedToolArgs]) &&
    stableStringify(payload.inputFingerprint) === stableStringify(identity.inputFingerprint);
}

function isMetricArray(value: unknown): value is DuplicateCodeFragment[] {
  if (!Array.isArray(value)) return false;
  return value.every(isDuplicateCodeFragment);
}

function isDuplicateCodeFragment(value: unknown): value is DuplicateCodeFragment {
  return isNonArrayRecord(value) &&
    isFiniteNumber(value.id) &&
    isFiniteNumber(value.tokenCount) &&
    isFiniteNumber(value.lineCount) &&
    typeof value.hitsChangedScope === "boolean" &&
    Array.isArray(value.codeAreas) &&
    value.codeAreas.every((area) => typeof area === "string") &&
    Array.isArray(value.locations) &&
    value.locations.every(isDuplicateCodeLocation);
}

function isDuplicateCodeLocation(value: unknown): value is DuplicateCodeLocation {
  return isNonArrayRecord(value) &&
    typeof value.path === "string" &&
    isFiniteNumber(value.startLine) &&
    isFiniteNumber(value.endLine) &&
    typeof value.codeArea === "string";
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function stableStringify(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }

  if (isNonArrayRecord(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`)
      .join(",")}}`;
  }

  return JSON.stringify(value);
}
