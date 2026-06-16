import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  checks,
  checksForProfile,
  formatCompletionLine,
  formatDurationMs,
  parseArgs,
  visibleOutputLines
} from "./verify-docnav-workspace.mjs";

describe("workspace verifier configuration", () => {
  it("filters successful node test runner output from release package script tests", () => {
    const check = checkByLabel("release package script tests");
    const output = [
      "  \u2714 package selection defaults to the current host package (1.6661ms)",
      "  \u2714 package selection accepts a target (0.2197ms)",
      "  \u2139 tests 5",
      "  \u2139 suites 0",
      "  \u2139 pass 5",
      "  \u2139 fail 0",
      "  \u2139 duration_ms 113.1451"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output), []);
  });

  it("keeps actionable output after filtering known success noise", () => {
    const check = checkByLabel("release package script tests");
    const output = [
      "  \u2714 package selection accepts a target (0.2197ms)",
      "unexpected diagnostic"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output), ["unexpected diagnostic"]);
  });

  it("separates required and full verification profiles", () => {
    const requiredLabels = checksForProfile(PROFILE_REQUIRED).map((check) => check.label);
    const fullLabels = checksForProfile(PROFILE_FULL).map((check) => check.label);

    assert.ok(requiredLabels.includes("cargo fmt"));
    assert.ok(requiredLabels.includes("docs schema validator"));
    assert.ok(requiredLabels.includes("git diff whitespace"));
    assert.ok(!requiredLabels.includes("cargo test"));
    assert.ok(!requiredLabels.includes("quality report tests"));
    assert.ok(!requiredLabels.includes("docnav core development smoke"));

    assert.ok(fullLabels.includes("cargo fmt"));
    assert.ok(fullLabels.includes("quality report tests"));
    assert.ok(fullLabels.includes("cargo test"));
    assert.ok(fullLabels.includes("docnav development binaries"));
    assert.ok(fullLabels.includes("docnav core development smoke"));
    assert.ok(fullLabels.includes("openspec"));
  });

  it("defines checks with explicit type, dependencies, and mutex metadata", () => {
    for (const check of checks) {
      assert.equal(typeof check.id, "string");
      assert.equal(typeof check.label, "string");
      assert.ok(check.type === PROFILE_REQUIRED || check.type === PROFILE_FULL);
      assert.ok(Array.isArray(check.dependsOn));
      assert.ok(Array.isArray(check.mutex));
      assert.equal(check.profiles, undefined);
    }

    assert.equal(checkByLabel("cargo test").type, PROFILE_FULL);
    assert.deepEqual(checkByLabel("cargo test").mutex, ["cargo-build"]);
    assert.deepEqual(checkByLabel("docnav development binaries").mutex, ["cargo-build"]);
    assert.deepEqual(checkByLabel("docnav-markdown development smoke").mutex, []);
    assert.deepEqual(checkByLabel("docnav core development smoke").mutex, []);
    assert.deepEqual(checkByLabel("docnav-markdown development smoke").dependsOn, ["docnav-development-binaries"]);
    assert.deepEqual(checkByLabel("docnav core development smoke").dependsOn, ["docnav-development-binaries"]);
    assert.equal(checkByLabel("docnav-markdown development smoke").envFile, ".log/verify-docnav-workspace/dev-bins.json");
    assert.deepEqual(checkByLabel("docs schema validator").dependsOn, []);
  });

  it("parses verification profile arguments", () => {
    assert.deepEqual(parseArgs([]), { help: false, profile: PROFILE_FULL });
    assert.deepEqual(parseArgs(["--profile", PROFILE_REQUIRED]), { help: false, profile: PROFILE_REQUIRED });
    assert.deepEqual(parseArgs([`--profile=${PROFILE_REQUIRED}`]), { help: false, profile: PROFILE_REQUIRED });
    assert.deepEqual(parseArgs(["--help"]), { help: true, profile: PROFILE_FULL });
    assert.throws(() => parseArgs(["--profile", "fast"]), /unknown verification profile: fast/);
  });

  it("formats completion lines and durations for streaming output", () => {
    assert.equal(formatDurationMs(234), "234ms");
    assert.equal(formatDurationMs(1250), "1.3s");
    assert.equal(formatDurationMs(65_000), "1m 05s");
    assert.equal(
      formatCompletionLine({
        ok: true,
        check: { label: "docs schema validator" },
        durationMs: 1250
      }),
      "  passed: docs schema validator (1.3s)"
    );
    assert.equal(
      formatCompletionLine({
        ok: false,
        check: { label: "cargo test" },
        durationMs: 65_000
      }),
      "  failed: cargo test (1m 05s)"
    );
  });

});

function checkByLabel(label) {
  const check = checks.find((candidate) => candidate.label === label);
  assert.ok(check, `expected check ${label}`);
  return check;
}
