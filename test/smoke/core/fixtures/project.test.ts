import { after, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import {
  configFixtureProject,
  mutableConfigFixtureProject
} from "./project.ts";
import { root, tempRoot } from "../config.ts";

after(() => {
  fs.rmSync(tempRoot, { recursive: true, force: true });
});

// @case AUX-SMOKE-HARNESS-002
describe("core smoke fixture projects", () => {
  it("uses semantic config fixtures with the shared Markdown document", () => {
    const project = configFixtureProject("project-native-option-outline");

    assert.equal(project.root.includes(tempRoot), true);
    assert.equal(fs.existsSync(path.join(project.docnavDir, "docnav.json")), true);
    assert.equal(fs.existsSync(project.normalPath), true);
    assert.equal(project.normalPath, path.join(root, "test", "smoke", "core", "fixtures", "normal.md"));
    assert.equal(project.normalRelPath, project.normalPath.replaceAll(path.sep, "/"));
  });

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

    assert.equal(project.root.includes(tempRoot), true);
    assert.equal(fs.readFileSync(sourceConfig, "utf8"), sourceContents);
    assert.notEqual(fs.readFileSync(copiedConfig, "utf8"), sourceContents);
  });
});
