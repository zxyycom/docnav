import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import { classifyFile, classifyFiles, isExcluded, buildFingerprint } from "../../scripts/quality/classify.mjs";

describe("file classification", () => {
  const { codeAreas } = DEFAULT_CONFIG;
  const generatedGlobs = DEFAULT_CONFIG.generatedFiles;

  it("classifies Rust production source to rust-production", () => {
    assert.equal(classifyFile("crates/docnav/src/lib.rs", codeAreas, generatedGlobs), "rust-production");
    assert.equal(classifyFile("crates/docnav-markdown/src/parser.rs", codeAreas, generatedGlobs), "rust-production");
  });

  it("classifies Rust tests to rust-tests", () => {
    assert.equal(classifyFile("crates/docnav/tests/integration_test.rs", codeAreas, generatedGlobs), "rust-tests");
  });

  it("classifies generated files to generated", () => {
    assert.equal(classifyFile("scripts/validators/generated/schema.json", codeAreas, generatedGlobs), "generated");
    assert.equal(classifyFile("crates/docnav/src/generated/mod.rs", codeAreas, generatedGlobs), "generated");
  });

  it("classifies Node smoke scripts to node-validation-smoke", () => {
    assert.equal(classifyFile("scripts/cli-smoke/index.mjs", codeAreas, generatedGlobs), "node-validation-smoke");
  });

  it("classifies validator scripts to node-validation-smoke", () => {
    assert.equal(classifyFile("scripts/validators/schema.mjs", codeAreas, generatedGlobs), "node-validation-smoke");
  });

  it("classifies Node production scripts", () => {
    assert.equal(classifyFile("scripts/cargo.mjs", codeAreas, generatedGlobs), "node-production-scripts");
  });

  it("classifies quality scripts and tests to validation/smoke area", () => {
    assert.equal(classifyFile("scripts/quality/scan.mjs", codeAreas, generatedGlobs), "node-validation-smoke");
    assert.equal(classifyFile("scripts/quality/warnings.mjs", codeAreas, generatedGlobs), "node-validation-smoke");
    assert.equal(classifyFile("test/quality/config-schema.test.mjs", codeAreas, generatedGlobs), "node-validation-smoke");
  });

  it("classifies fixtures to fixtures-examples", () => {
    assert.equal(classifyFile("test/fixtures/quality/sample.rs", codeAreas, generatedGlobs), "fixtures-examples");
  });

  it("isExcluded detects build artifacts", () => {
    assert.ok(isExcluded("target/debug/build/output.rs", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
    assert.ok(isExcluded("node_modules/some-pkg/index.js", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
    assert.ok(isExcluded(".git/objects/abc", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
    assert.ok(isExcluded("dist/bundle.js", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
  });

  it("isExcluded does not flag normal source", () => {
    assert.ok(!isExcluded("crates/docnav/src/lib.rs", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
    assert.ok(!isExcluded("scripts/cargo.mjs", DEFAULT_CONFIG.excludeDirs, generatedGlobs));
  });

  it("classifyFiles returns grouped map", () => {
    const files = [
      "crates/docnav/src/lib.rs",
      "crates/docnav/tests/test.rs",
      "scripts/cargo.mjs",
      "scripts/cli-smoke/index.mjs",
      "test/fixtures/sample.rs"
    ];
    const result = classifyFiles(files, codeAreas, generatedGlobs);

    assert.ok(result instanceof Map);

    // Check that each file is classified
    let totalClassified = 0;
    for (const files of result.values()) {
      totalClassified += files.length;
    }
    assert.equal(totalClassified, files.length);
  });

  it("does not silently classify included default files as unknown", () => {
    const files = [
      "crates/docnav/src/lib.rs",
      "crates/docnav/tests/integration_test.rs",
      "scripts/cargo.mjs",
      "scripts/quality/scan.mjs",
      "test/quality/config-schema.test.mjs",
      "test/fixtures/sample.rs"
    ];
    const result = classifyFiles(files, codeAreas, generatedGlobs);

    assert.equal(result.has("unknown"), false);
  });

  it("buildFingerprint produces consistent output", () => {
    const files = ["a.rs", "b.rs", "c.mjs"];
    const finger1 = buildFingerprint("test-area", files, () => "hash123");
    const finger2 = buildFingerprint("test-area", files, () => "hash123");

    assert.equal(finger1.fingerprint, finger2.fingerprint);
    assert.equal(finger1.fileCount, 3);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Warning 生成测试
// ═══════════════════════════════════════════════════════════════════════

describe("path filtering and normalization", () => {
  it("excludes .git directory", () => {
    assert.ok(isExcluded(".git/config", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
    assert.ok(isExcluded("crates/.git/HEAD", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
  });

  it("excludes target directory", () => {
    assert.ok(isExcluded("target/debug/docnav.exe", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
    assert.ok(isExcluded("crates/docnav/target/release/lib.rlib", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
  });

  it("excludes node_modules", () => {
    assert.ok(isExcluded("node_modules/something/index.js", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
  });

  it("excludes cache directories", () => {
    assert.ok(isExcluded("__pycache__/module.pyc", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
    assert.ok(isExcluded(".ruff_cache/something", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
    assert.ok(isExcluded(".pnpm-store/v3/files/abc", DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
  });

  it("excludes generated file globs", () => {
    assert.ok(isExcluded("scripts/validators/generated/schema.json",
      DEFAULT_CONFIG.excludeDirs, DEFAULT_CONFIG.generatedFiles));
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Baseline commit 和 comparison status 测试
// ═══════════════════════════════════════════════════════════════════════
