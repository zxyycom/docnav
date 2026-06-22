import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { classifyFile } from "./code-areas.ts";
import { DEFAULT_CONFIG } from "./config.ts";

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

  it("keeps source scan globs on TypeScript script sources", () => {
    assert.deepEqual(DEFAULT_CONFIG.include, [
      "crates/**/*.rs",
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
});

function classifyQualityFile(filePath: string): string {
  return classifyFile(filePath, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles);
}
