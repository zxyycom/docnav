import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { submoduleStatusFailures } from "./index.ts";

describe("project workspace environment", () => {
  it("starts without shared script submodules", () => {
    const tempRoot = mkdtempSync(join(tmpdir(), "docnav-project-environment-"));
    const isolatedEntry = join(tempRoot, "index.ts");

    try {
      copyFileSync(fileURLToPath(new URL("./index.ts", import.meta.url)), isolatedEntry);
      const result = spawnSync(process.execPath, [isolatedEntry, "invalid"], {
        encoding: "utf8"
      });

      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /usage: bun scripts\/project-environment\/index\.ts <check\|setup>/u);
    } finally {
      rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it("accepts pinned submodules and rejects unavailable or mismatched revisions", () => {
    const pinned = [
      " 1111111111111111111111111111111111111111 scripts/tools/foundation (heads/main)",
      " 2222222222222222222222222222222222222222 subrepos/cli-config-resolution (heads/main)"
    ].join("\n");
    const invalid = [
      "-1111111111111111111111111111111111111111 scripts/tools/foundation",
      "+2222222222222222222222222222222222222222 subrepos/cli-config-resolution (heads/main)",
      "U3333333333333333333333333333333333333333 nested/conflicted"
    ].join("\n");

    assert.deepEqual(submoduleStatusFailures(pinned), []);
    assert.deepEqual(submoduleStatusFailures(invalid), invalid.split("\n"));
  });
});
