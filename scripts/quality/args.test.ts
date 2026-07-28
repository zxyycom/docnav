import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { parseArgs } from "./args.ts";

describe("quality scan CLI args", () => {
  it("skips baseline by default and keeps baseline generation opt-in", () => {
    assert.deepEqual(profileAndBaseline(parseArgs([])), {
      baseline: null,
      scanProfile: "full",
      skipBaseline: true
    });
    assert.deepEqual(profileAndBaseline(parseArgs(["--with-baseline"])), {
      baseline: null,
      scanProfile: "full",
      skipBaseline: false
    });
    assert.deepEqual(profileAndBaseline(parseArgs(["--baseline", "abc123"])), {
      baseline: "abc123",
      scanProfile: "full",
      skipBaseline: false
    });
  });

  it("keeps quick quality checks baseline-free and explicit", () => {
    assert.deepEqual(profileAndBaseline(parseArgs(["--profile", "quick"])), {
      baseline: null,
      scanProfile: "quick",
      skipBaseline: true
    });
    assert.equal(parseArgs(["--profile", "full", "--with-baseline"]).skipBaseline, false);
    assert.throws(
      () => parseArgs(["--profile", "quick", "--with-baseline"]),
      /quick quality check does not support baseline/
    );
    assert.throws(() => parseArgs(["--profile", "fast"]), /unknown quality scan profile: fast/);
  });
});

function profileAndBaseline(result: ReturnType<typeof parseArgs>) {
  return {
    baseline: result.baseline,
    scanProfile: result.scanProfile,
    skipBaseline: result.skipBaseline
  };
}
