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
    assert.equal(classifyQualityFile("test/tools/smoke-harness.ts"), "node-validation-smoke");
    assert.equal(classifyQualityFile("scripts/tools/validators/schema/index.ts"), "node-validation-smoke");
  });
});

function classifyQualityFile(filePath: string): string {
  return classifyFile(filePath, DEFAULT_CONFIG.codeAreas, DEFAULT_CONFIG.generatedFiles);
}
