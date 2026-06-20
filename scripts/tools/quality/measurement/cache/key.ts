import { createHash } from "node:crypto";
import { join } from "node:path";

import { isNonArrayRecord } from "../../../type-guards.ts";
import {
  SCAN_CACHE_VERSION,
  type CpdCacheIdentity
} from "./types.ts";

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

export function stableStringify(value: unknown): string {
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
