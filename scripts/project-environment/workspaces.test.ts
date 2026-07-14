import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { submoduleStatusFailures } from "./index.ts";

describe("project workspace environment", () => {
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
