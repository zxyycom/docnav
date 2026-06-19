import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "./config.ts";
import { parseArgs, resolveChangedFilesForScan } from "./scan-cli.ts";

// @case AUX-QUALITY-SCAN-CLI-001
describe("quality scan CLI args", () => {
  it("skips baseline by default and keeps baseline generation opt-in", () => {
    const defaults = parseArgs([]);

    assert.deepEqual(defaults, {
      artifactDir: DEFAULT_CONFIG.artifactDir,
      baseline: null,
      changedFiles: null,
      skipBaseline: true,
      topN: DEFAULT_CONFIG.report.topN
    });
    assert.equal(parseArgs(["--with-baseline"]).skipBaseline, false);
    assert.deepEqual(parseArgs(["--baseline", "abc123"]), {
      artifactDir: DEFAULT_CONFIG.artifactDir,
      baseline: "abc123",
      changedFiles: null,
      skipBaseline: false,
      topN: DEFAULT_CONFIG.report.topN
    });
    const changedFiles = resolveChangedFilesForScan({
      opts: defaults,
      root: "/repo",
      scope: { changed: true, changedFiles: [] },
      collectChangedFiles: () => ["scripts/quality-scan.ts"]
    });

    assert.deepEqual(changedFiles, ["scripts/quality-scan.ts"]);
  });
});
