import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { createEmptyMetrics, validateMetrics } from "../../model/schema.ts";
import type { ScanContext } from "./scan-context.ts";
import { runCurrentRevisionScan } from "./index.ts";
import { TEST_QUALITY_CONFIG } from "../../../test/config.ts";

describe("current revision duplicate-code measurement", () => {
  it("records measured, profile-skipped, unavailable, and error states", async () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-current-"));
    try {
      const quick = scanContext(tempDir, []);
      await withMutedConsoleLog(() => runCurrentRevisionScan({
        context: quick,
        fileMap: new Map(),
        scanFiles: [],
        scanProfile: "quick"
      }));
      assert.equal(quick.metrics.duplicateCodeMeasurement.status, "skipped-by-profile");

      const measured = scanContext(tempDir, [availableJscpd()]);
      await withMutedConsoleLog(() => runCurrentRevisionScan({
        context: measured,
        fileMap: new Map(),
        scanFiles: [],
        scanProfile: "full"
      }));
      assert.equal(measured.metrics.duplicateCodeMeasurement.status, "measured");
      assert.deepEqual(measured.metrics.duplicateCode, []);

      const unavailable = scanContext(tempDir, [unavailableJscpd("tool-unavailable")]);
      await withMutedConsoleLog(() => runCurrentRevisionScan({
        context: unavailable,
        fileMap: new Map(),
        scanFiles: [],
        scanProfile: "full"
      }));
      assert.equal(unavailable.metrics.duplicateCodeMeasurement.status, "unavailable");

      const error = scanContext(tempDir, [unavailableJscpd("execution-error")]);
      await withMutedConsoleLog(() => runCurrentRevisionScan({
        context: error,
        fileMap: new Map(),
        scanFiles: [],
        scanProfile: "full"
      }));
      assert.equal(error.metrics.duplicateCodeMeasurement.status, "error");

      for (const context of [quick, measured, unavailable, error]) {
        assert.equal(validateMetrics(context.metrics).valid, true);
      }
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

function scanContext(root: string, toolResults: ScanContext["toolResults"]): ScanContext {
  return {
    cacheRootDir: join(root, "cache"),
    changedFiles: [],
    config: TEST_QUALITY_CONFIG,
    fatalIssues: [],
    fingerprints: {},
    metrics: createEmptyMetrics({
      configVersion: TEST_QUALITY_CONFIG.version,
      commitSha: "abc123",
      repository: root,
      scope: { excludeDirs: [], generatedFiles: [], include: [] },
      tools: []
    }),
    rawDir: join(root, "raw"),
    root,
    toolResults
  };
}

function availableJscpd(): ScanContext["toolResults"][number] {
  return {
    available: true,
    error: null,
    name: "jscpd",
    reason: null,
    source: "repository devDependency",
    version: "5.0.11"
  };
}

function unavailableJscpd(reason: "tool-unavailable" | "execution-error"): ScanContext["toolResults"][number] {
  return {
    available: false,
    error: reason,
    name: "jscpd",
    reason,
    source: "repository devDependency",
    version: null
  };
}

async function withMutedConsoleLog<T>(callback: () => Promise<T>): Promise<T> {
  const originalLog = console.log;
  console.log = () => undefined;
  try {
    return await callback();
  } finally {
    console.log = originalLog;
  }
}
