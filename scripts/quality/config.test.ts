import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { DEFAULT_CONFIG } from "./config.ts";
import { collectScanFiles } from "../tools/quality-core/src/index.ts";
import { classifyFile } from "../tools/quality-core/src/model/code-areas.ts";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

// @case AUX-QUALITY-CODE-AREAS-001
describe("quality code area classification", () => {
  it("keeps smoke case and fixture files in the fixtures/examples area", () => {
    assert.equal(classifyQualityFile("test/smoke/core/cases/config-management.ts"), "fixtures-examples");
    assert.equal(classifyQualityFile("test/smoke/core/fixtures/project.ts"), "fixtures-examples");
  });

  it("keeps smoke harness infrastructure in the validation/smoke area", () => {
    assert.equal(classifyQualityFile("test/tools/smoke-harness.ts"), "typescript-validation-smoke");
    assert.equal(classifyQualityFile("scripts/tools/validators/schema/index.ts"), "typescript-validation-smoke");
  });

  it("classifies nested workspace crates by Rust source role", () => {
    assert.equal(classifyQualityFile("crates/shared/protocol/src/lib.rs"), "rust-production");
    assert.equal(classifyQualityFile("crates/shared/protocol/src/tests/schema.rs"), "rust-tests");
    assert.equal(classifyQualityFile("crates/shared/readable/src/renderer/tests/success.rs"), "rust-tests");
    assert.equal(classifyQualityFile("crates/adapters/markdown/tests/adapter.rs"), "rust-tests");
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/src/lib.rs"),
      "rust-production"
    );
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/src/tests/constraints.rs"),
      "rust-tests"
    );
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/tests/canonical_parameters.rs"),
      "rust-tests"
    );
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/benches/resolution.rs"),
      "rust-tests"
    );
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/cli-config-resolution-clap/examples/resolution_flow.rs"),
      "fixtures-examples"
    );
  });

  it("discovers representative Rust and TypeScript sources across workspaces", () => {
    const files = collectScanFiles(REPO_ROOT, DEFAULT_CONFIG);

    for (const file of [
      "crates/shared/protocol/src/lib.rs",
      "subrepos/cli-config-resolution/crates/typed-fields/src/lib.rs",
      "scripts/quality/scan.ts",
      "test/tools/smoke-harness.ts"
    ]) {
      assert.ok(files.includes(file), `quality current scan should include ${file}`);
    }
  });

});

function classifyQualityFile(filePath: string): string {
  return classifyFile(filePath, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles);
}
