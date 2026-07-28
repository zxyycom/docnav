import { after, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { mutableConfigFixtureProject } from "./project.ts";
import { root, tempRoot } from "../config.ts";

after(() => {
  fs.rmSync(tempRoot, { recursive: true, force: true });
});

describe("core smoke fixture projects", () => {
  it("copies config fixtures before mutable config cases write", () => {
    const project = mutableConfigFixtureProject("config-precedence-base", "mutable-config-copy");
    const sourceConfig = path.join(
      root,
      "test",
      "smoke",
      "core",
      "fixtures",
      "configs",
      "config-precedence-base.json"
    );
    const copiedConfig = path.join(project.docnavDir, "docnav.json");
    const sourceContents = fs.readFileSync(sourceConfig, "utf8");

    assert.equal(fs.readFileSync(copiedConfig, "utf8"), sourceContents);
    fs.writeFileSync(copiedConfig, "{}\n", "utf8");

    assert.equal(path.dirname(project.root), tempRoot);
    assert.equal(fs.readFileSync(sourceConfig, "utf8"), sourceContents);
    assert.notEqual(fs.readFileSync(copiedConfig, "utf8"), sourceContents);
  });
});
