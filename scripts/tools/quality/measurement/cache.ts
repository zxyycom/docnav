/**
 * Code-area cache for normalized PMD CPD duplicate-code results.
 */

import { readJsonFile, writeJsonFile } from "../../fs.ts";
import type { DuplicateCodeFragment } from "../model/schema.ts";
import { buildScanCacheKey, getScanCachePath } from "./cache/key.ts";
import { isMatchingPayload, isMetricArray, stripDuplicateChangedScope } from "./cache/payload.ts";
import {
  SCAN_CACHE_VERSION,
  type CpdCacheHit,
  type CpdCacheIdentity,
  type CpdCacheMiss,
  type ScanCachePayload
} from "./cache/types.ts";

export { buildScanCacheKey, getScanCachePath } from "./cache/key.ts";
export { SCAN_CACHE_VERSION } from "./cache/types.ts";
export type { CpdCacheHit, CpdCacheIdentity, CpdCacheMiss, ScanKind } from "./cache/types.ts";

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
