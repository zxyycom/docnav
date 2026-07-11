import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "./config.ts";
import { classifyFile, isExcluded } from "../tools/quality-core/src/model/code-areas.ts";

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

  it("keeps source scan globs on Rust and TypeScript sources", () => {
    assert.deepEqual(DEFAULT_CONFIG.include, [
      "crates/**/*.rs",
      "subrepos/cli-config-resolution/crates/**/*.rs",
      "scripts/**/*.ts",
      "test/**/*.ts"
    ]);
    assert.deepEqual(DEFAULT_CONFIG.codeAreas["typescript-production-scripts"].globs, [
      "scripts/**/*.ts"
    ]);
    assert.deepEqual(DEFAULT_CONFIG.codeAreas["typescript-validation-smoke"].globs, [
      "scripts/tools/validators/**/*.ts",
      "scripts/**/*.test.ts",
      "test/smoke/**/*.ts",
      "test/tools/**/*.ts",
      "test/**/*.ts"
    ]);
  });

  it("classifies nested workspace crates by Rust source role", () => {
    assert.equal(classifyQualityFile("crates/shared/protocol/src/lib.rs"), "rust-production");
    assert.equal(classifyQualityFile("crates/shared/protocol/src/tests/schema.rs"), "rust-tests");
    assert.equal(classifyQualityFile("crates/adapters/markdown/tests/adapter.rs"), "rust-tests");
  });

  it("classifies reusable nested workspace crates by Rust source role", () => {
    assert.deepEqual([
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/src/lib.rs"),
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/src/tests/processing.rs"),
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/tests/canonical_parameters.rs"),
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/benches/resolution.rs")
    ], ["rust-production", "rust-tests", "rust-tests", "rust-tests"]);

    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/cli-config-resolution-clap/examples/resolution_flow.rs"),
      "fixtures-examples"
    );
    assert.equal(
      classifyQualityFile("subrepos/cli-config-resolution/crates/typed-fields/tests/fixtures/invalid.rs"),
      "fixtures-examples"
    );
    assert.equal(
      isExcluded(
        "subrepos/cli-config-resolution/crates/typed-fields/target/debug/build/generated.rs",
        DEFAULT_CONFIG.excludeDirs,
        DEFAULT_CONFIG.generatedFiles
      ),
      true
    );
  });
});

function classifyQualityFile(filePath: string): string {
  return classifyFile(filePath, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles);
}
