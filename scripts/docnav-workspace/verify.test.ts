import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  createReportCompletionTracker,
  checks,
  checksForProfile,
  formatCompletionLine,
  formatDurationMs,
  parseArgs,
  printCompletionResult,
  reportCountForChecks,
  resolveVerificationConcurrency,
  type CheckResult,
  visibleOutputForCheck,
  visibleOutputLines
} from "./verify.ts";
import { executeCheck } from "./verify/execution.ts";

// @case AUX-WORKSPACE-VERIFY-001
describe("workspace verifier configuration", () => {
  it("filters successful test runner output from release package script tests", () => {
    const check = checkByLabel("release package script tests");
    const output = [
      "bun test v1.3.14 (0d9b296a)",
      "",
      "scripts\\tools\\release-package\\args.test.ts:",
      "(pass) package selection defaults to the current host package [0.27ms]",
      "(pass) package selection accepts a target [0.23ms]",
      "(pass) package build target accepts one target option [1.14s]",
      "",
      "  3 pass",
      "  0 fail",
      "Ran 3 tests across 1 file. [1.61s]"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output), []);
  });

  it("keeps actionable output after filtering known success noise", () => {
    const check = checkByLabel("release package script tests");
    const output = [
      "(pass) package selection accepts a target [0.23ms]",
      "unexpected diagnostic"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output), ["unexpected diagnostic"]);
  });

  it("filters package manager echoes from successful script checks", () => {
    const check = checkByLabel("TypeScript script typecheck");

    assert.deepEqual(visibleOutputLines(check, "$ tsgo -p tsconfig.json"), []);
  });

  it("filters cargo trybuild success noise from successful cargo test output", () => {
    const check = checkByLabel("cargo test");
    const output = [
      "running 1 test",
      "test \u001b[0m\u001b[1mtests/ui/field_defs_type_mismatch.rs\u001b[0m ... \u001b[0m\u001b[32mok",
      "\u001b[0mtest \u001b[0m\u001b[1mtests/ui/field_defs_missing_validation.rs\u001b[0m ... \u001b[0m\u001b[32mok",
      "\u001b[0m",
      "   Blocking waiting for file lock on package cache",
      "    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.33s"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output, "passed"), []);
  });

  it("carries visible child output into report completions", () => {
    const catalogCheck = checkByLabel("docs case catalog validator");
    const schemaCheck = checkByLabel("docs schema validator");
    const completeReport = createReportCompletionTracker([catalogCheck, schemaCheck]);

    assert.equal(
      completeReport(checkResult(catalogCheck, {
        stdout: [
          "test case catalog ok: 86 implemented, 0 planned",
          "catalog diagnostic"
        ].join("\n")
      })),
      null
    );

    const report = completeReport(checkResult(schemaCheck, {
      stderr: [
        "schema ok: docs/schemas/readable-read.schema.json",
        "schema diagnostic"
      ].join("\n")
    }));

    assert.ok(report);
    assert.equal(report.check.label, "docs validators");
    assert.equal(report.visibleOutput, "catalog diagnostic\nschema diagnostic");
    assert.equal(report.combinedOutput, report.visibleOutput);
  });

  it("prints visible report output immediately after completion lines", () => {
    const lines: string[] = [];

    printCompletionResult(
      {
        status: "passed",
        check: { id: "docs-validators", label: "docs validators" },
        durationMs: 1250,
        visibleOutput: "catalog diagnostic\nschema diagnostic"
      },
      (line) => lines.push(line)
    );

    assert.deepEqual(lines, [
      "  passed: docs validators (1.3s)",
      "catalog diagnostic\nschema diagnostic"
    ]);
  });

  it("separates required and full verification profiles", () => {
    const requiredLabels = checksForProfile(PROFILE_REQUIRED).map((check) => check.label);
    const fullLabels = checksForProfile(PROFILE_FULL).map((check) => check.label);

    assert.ok(requiredLabels.includes("cargo fmt"));
    assert.ok(requiredLabels.includes("TypeScript script typecheck"));
    assert.ok(requiredLabels.includes("TypeScript script lint"));
    assert.ok(requiredLabels.includes("quality quick check"));
    assert.ok(requiredLabels.includes("docs case catalog validator"));
    assert.ok(requiredLabels.includes("docs schema validator"));
    assert.ok(requiredLabels.includes("case catalog validator tests"));
    assert.ok(requiredLabels.includes("smoke harness tests"));
    assert.ok(requiredLabels.includes("git diff whitespace"));
    assert.ok(!requiredLabels.includes("cargo test"));
    assert.ok(!requiredLabels.includes("quality internal script tests"));
    assert.ok(!requiredLabels.includes("docnav core development smoke"));

    assert.ok(fullLabels.includes("cargo fmt"));
    assert.ok(!fullLabels.includes("quality quick check"));
    assert.ok(fullLabels.includes("quality full check"));
    assert.ok(fullLabels.includes("quality internal script tests"));
    assert.ok(!fullLabels.includes("quality report tests"));
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
      assert.equal((check as { profiles?: unknown }).profiles, undefined);
    }

    assert.equal(checkByLabel("cargo test").type, PROFILE_FULL);
    assert.equal(checkByLabel("quality quick check").type, PROFILE_REQUIRED);
    assert.equal(checkByLabel("quality quick check").command, "bun");
    assert.deepEqual(checkByLabel("quality quick check").args, [
      "scripts/quality/scan.ts",
      "--profile",
      "quick",
      "--artifact-dir",
      "artifacts/docnav-quality/quick"
    ]);
    assert.deepEqual(checkByLabel("quality quick check").warningOutput, [/^Quality check status: warning$/m]);
    assert.equal(checkByLabel("quality full check").type, PROFILE_FULL);
    assert.equal(checkByLabel("quality full check").command, "bun");
    assert.deepEqual(checkByLabel("quality full check").args, [
      "scripts/quality/scan.ts",
      "--profile",
      "full",
      "--with-baseline"
    ]);
    assert.deepEqual(checkByLabel("quality full check").warningOutput, [/^Quality check status: warning$/m]);
    assert.deepEqual(checkByLabel("cargo test").mutex, ["cargo-build"]);
    assert.deepEqual(checkByLabel("docnav development binaries").mutex, ["cargo-build"]);
    assert.deepEqual(checkByLabel("docnav-markdown development smoke").mutex, []);
    assert.deepEqual(checkByLabel("docnav core development smoke").mutex, []);
    assert.deepEqual(checkByLabel("docnav-markdown development smoke").dependsOn, ["docnav-development-binaries"]);
    assert.deepEqual(checkByLabel("docnav core development smoke").dependsOn, ["docnav-development-binaries"]);
    assert.equal(checkByLabel("docnav-markdown development smoke").envFile, ".cache/docnav/verify/dev-bins.json");
    assert.deepEqual(checkByLabel("docs case catalog validator").dependsOn, []);
    assert.deepEqual(checkByLabel("docs schema validator").dependsOn, []);
    assert.deepEqual(checkByLabel("case catalog validator tests").dependsOn, []);
  });

  it("parses verification profile arguments", () => {
    assert.deepEqual(parseArgs([]), { help: false, profile: PROFILE_FULL, concurrency: undefined });
    assert.deepEqual(parseArgs(["--profile", PROFILE_REQUIRED]), {
      help: false,
      profile: PROFILE_REQUIRED,
      concurrency: undefined
    });
    assert.deepEqual(parseArgs([`--profile=${PROFILE_REQUIRED}`]), {
      help: false,
      profile: PROFILE_REQUIRED,
      concurrency: undefined
    });
    assert.deepEqual(parseArgs(["--concurrency", "2"]), { help: false, profile: PROFILE_FULL, concurrency: 2 });
    assert.deepEqual(parseArgs(["--concurrency=3"]), { help: false, profile: PROFILE_FULL, concurrency: 3 });
    assert.deepEqual(parseArgs(["--help"]), { help: true, profile: PROFILE_FULL, concurrency: undefined });
    assert.throws(() => parseArgs(["--profile", "fast"]), /unknown verification profile: fast/);
    assert.throws(() => parseArgs(["--concurrency", "0"]), /positive integer/);
  });

  it("resolves verifier concurrency only when a limit is configured", () => {
    assert.equal(resolveVerificationConcurrency(undefined), undefined);
    assert.equal(resolveVerificationConcurrency(""), undefined);
    assert.equal(resolveVerificationConcurrency("8"), 8);
    assert.throws(() => resolveVerificationConcurrency("abc"), /positive integer/);
  });

  it("formats completion lines and durations for streaming output", () => {
    assert.equal(formatDurationMs(234), "234ms");
    assert.equal(formatDurationMs(1250), "1.3s");
    assert.equal(formatDurationMs(65_000), "1m 05s");
    assert.equal(
      formatCompletionLine({
        status: "passed",
        check: { id: "docs-schema-validator", label: "docs schema validator" },
        durationMs: 1250
      }),
      "  passed: docs schema validator (1.3s)"
    );
    assert.equal(
      formatCompletionLine({
        status: "failed",
        check: { id: "cargo-test", label: "cargo test" },
        durationMs: 65_000
      }),
      "  failed: cargo test (1m 05s)"
    );
    assert.equal(
      formatCompletionLine({
        status: "warning",
        check: { id: "quality-quick-check", label: "quality quick check" },
        durationMs: 987
      }),
      "  warning: quality quick check (987ms)"
    );
  });

  it("maps quality warning markers to warning check status", async () => {
    const result = await executeCheck({
      id: "quality-warning-marker-test",
      label: "quality warning marker test",
      type: PROFILE_REQUIRED,
      command: "bun",
      args: ["-e", "console.log('Quality check status: warning')"],
      dependsOn: [],
      mutex: [],
      ignoreOutput: [],
      warningOutput: [/^Quality check status: warning$/m]
    });

    assert.equal(result.ok, true);
    assert.equal(result.status, "warning");
  });

  it("counts top-level report groups separately from executable leaf checks", () => {
    const requiredChecks = checksForProfile(PROFILE_REQUIRED);

    assert.ok(requiredChecks.some((check) => check.label === "docs case catalog validator"));
    assert.ok(requiredChecks.some((check) => check.label === "docs schema validator"));
    assert.ok(!requiredChecks.some((check) => check.label === "docs validators"));
    assert.equal(reportCountForChecks(requiredChecks), 10);
  });

});

function checkByLabel(label: string) {
  const check = checks.find((candidate) => candidate.label === label);
  assert.ok(check, `expected check ${label}`);
  return check;
}

type TestCheckTask = (typeof checks)[number];

function checkResult(
  check: TestCheckTask,
  { stdout = "", stderr = "" }: { stdout?: string; stderr?: string }
): CheckResult {
  const combinedOutput = [stdout, stderr].filter(Boolean).join("\n");
  return {
    check,
    ok: true,
    exitCode: 0,
    error: null,
    status: "passed",
    stdout,
    stderr,
    combinedOutput,
    visibleOutput: visibleOutputForCheck(check, combinedOutput),
    durationMs: 1,
    startedAtMs: 1,
    endedAtMs: 2
  };
}
