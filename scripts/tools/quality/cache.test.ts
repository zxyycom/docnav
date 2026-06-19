import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  buildScanCacheKey,
  loadScanCacheEntry,
  writeScanCacheEntry,
  type CpdCacheIdentity
} from "./cache.ts";
import type { DuplicateCodeFragment } from "./schema.ts";

// @case AUX-QUALITY-CACHE-001
describe("quality CPD cache", () => {
  it("keys duplicate-code cache by scan identity and strips changed-scope annotations", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-cache-"));
    const identity = cacheIdentity();
    const fragment = duplicateFragment();
    fragment.hitsChangedScope = true;

    try {
      const baseKey = buildScanCacheKey(identity);
      assert.notEqual(baseKey, buildScanCacheKey({ ...identity, codeArea: "rust-tests" }));
      assert.notEqual(
        baseKey,
        buildScanCacheKey({
          ...identity,
          inputFingerprint: {
            fileCount: 1,
            fileList: ["src/changed.ts"],
            fingerprint: "sha256:changed:1"
          }
        })
      );

      writeScanCacheEntry({ rootDir: tempDir, identity, metrics: [fragment] });

      const hit = loadScanCacheEntry({ rootDir: tempDir, identity });
      assert.equal(hit.hit, true);
      assert.equal(hit.hit ? hit.metrics[0]!.hitsChangedScope : true, false);

      const mismatched = loadScanCacheEntry({
        rootDir: tempDir,
        identity: { ...identity, toolVersion: "7.26.0" }
      });
      assert.equal(mismatched.hit, false);
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

function cacheIdentity(): CpdCacheIdentity {
  return {
    scanKind: "current",
    toolName: "pmd-cpd",
    toolVersion: "7.25.0",
    normalizedToolArgs: ["cpd", "--minimum-tokens", "75", "--format", "xml"],
    configVersion: "quality-observability-v1",
    codeArea: "node-production-scripts",
    commitSha: "abc123",
    inputFingerprint: {
      fileCount: 1,
      fileList: ["src/risky.ts"],
      fingerprint: "sha256:test:1"
    }
  };
}

function duplicateFragment(): DuplicateCodeFragment {
  return {
    id: 1,
    tokenCount: 90,
    lineCount: 10,
    codeAreas: [],
    hitsChangedScope: false,
    locations: [
      { path: "src/a.ts", startLine: 10, endLine: 20, codeArea: "unknown" },
      { path: "src/b.ts", startLine: 11, endLine: 21, codeArea: "unknown" }
    ]
  };
}
