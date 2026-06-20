import type {
  CodeAreaFingerprint,
  DuplicateCodeFragment
} from "../../model/schema.ts";

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

export type ScanCachePayload = {
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
