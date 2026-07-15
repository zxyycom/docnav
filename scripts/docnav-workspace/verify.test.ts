import { describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  cleanupDevBinArtifacts,
  prepareDevBinEnv,
  type DevBinarySpec
} from "../docnav-dev/build-bins.ts";
import {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  checks,
  checksForProfile,
  formatCompletionLine,
  formatDurationMs,
  parseArgs,
  printCompletionResult,
  reportCountForChecks,
  resolveVerificationConcurrency,
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
    const typecheck = checkByLabel("TypeScript script typecheck");
    const lint = checkByLabel("TypeScript script lint");

    assert.deepEqual(visibleOutputLines(typecheck, "$ tsgo -p tsconfig.json"), []);
    assert.deepEqual(
      visibleOutputLines(
        lint,
        "$ eslint --max-warnings 0 --cache --cache-location .eslintcache --cache-strategy content"
      ),
      []
    );
  });

  it("filters docs validator success details", () => {
    const check = checkByLabel("docs validators");

    assert.deepEqual(visibleOutputLines(check, "readable error details shape ok", "passed"), []);
  });

  it("filters workspace verifier script test package output", () => {
    const check = checkByLabel("workspace verifier script tests");
    const output = [
      "$ bun test scripts/docnav-workspace/verify.test.ts scripts/project-environment/workspaces.test.ts test/tools/smoke-harness.test.ts test/smoke/core/fixtures/project.test.ts scripts/tools/foundation/test/foundation.test.ts scripts/tools/parallel-task-runner/test/index.test.ts",
      "bun test v1.3.14 (0d9b296a)",
      "",
      "scripts\\docnav-workspace\\verify.test.ts:",
      "(pass) workspace verifier configuration > filters output [0.27ms]",
      "",
      "  1 pass",
      "  0 fail",
      "Ran 1 test across 1 file. [1.61s]"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output, "passed"), []);
  });

  it("filters quality timing details from terminal-visible output", () => {
    const check = checkByLabel("quality full check");
    const output = [
      "Quality verification status: passed",
      "",
      "Timing breakdown:",
      "  123ms  scan current revision",
      "  456ms  total"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output, "passed"), []);
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
    assert.ok(requiredLabels.includes("docs validators"));
    assert.ok(!requiredLabels.includes("docs case catalog validator"));
    assert.ok(!requiredLabels.includes("docs schema validator"));
    assert.ok(requiredLabels.includes("workspace verifier script tests"));
    assert.ok(requiredLabels.includes("case catalog validator tests"));
    assert.ok(requiredLabels.includes("git diff whitespace"));
    assert.ok(!requiredLabels.includes("cargo test"));
    assert.ok(!requiredLabels.includes("quality internal tests"));
    assert.ok(!requiredLabels.includes("docnav core development smoke"));

    assert.ok(fullLabels.includes("cargo fmt"));
    assert.ok(!fullLabels.includes("quality quick check"));
    assert.ok(fullLabels.includes("quality full check"));
    assert.ok(fullLabels.includes("quality internal tests"));
    assert.ok(!fullLabels.includes("quality report tests"));
    assert.ok(fullLabels.includes("cargo test"));
    assert.ok(fullLabels.includes("docnav development binaries"));
    assert.ok(fullLabels.includes("docnav core development smoke"));
    assert.ok(fullLabels.includes("openspec"));
  });

  it("parses verification profile arguments", () => {
    assert.deepEqual(parseArgs([]), { help: false, profile: PROFILE_FULL, concurrency: undefined });
    assert.deepEqual(parseArgs(["--profile", PROFILE_REQUIRED]), {
      help: false,
      profile: PROFILE_REQUIRED,
      concurrency: undefined
    });
    assert.deepEqual(parseArgs(["--concurrency", "2"]), { help: false, profile: PROFILE_FULL, concurrency: 2 });
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

  it("prepares development binary env with isolated copied executables", () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-dev-bins-"));
    try {
      const sourceDir = path.join(tempRoot, "target-debug");
      const copyRoot = path.join(tempRoot, "copies");
      fs.mkdirSync(sourceDir, { recursive: true });

      const docnavSource = path.join(sourceDir, executableName("docnav"));
      fs.writeFileSync(docnavSource, "docnav");

      const specs: DevBinarySpec[] = [
        { packageName: "docnav", binName: "docnav", envName: "DOCNAV_BIN" }
      ];
      const env = prepareDevBinEnv({
        binaries: specs,
        copyTo: copyRoot,
        executables: new Map([
          ["docnav", docnavSource]
        ])
      });

      assert.notEqual(env.DOCNAV_BIN, docnavSource);
      assert.match(path.relative(copyRoot, env.DOCNAV_BIN), /^run-[^\\/]+[\\/]docnav(?:\.exe)?$/);
      assert.equal(fs.readFileSync(env.DOCNAV_BIN, "utf8"), "docnav");
    } finally {
      fs.rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it("removes copied development binary artifacts", () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-dev-bins-cleanup-"));
    try {
      const copyRoot = path.join(tempRoot, "copies");
      const envFile = path.join(tempRoot, "dev-bins.json");
      fs.mkdirSync(path.join(copyRoot, "run-example"), { recursive: true });
      fs.writeFileSync(envFile, "{}");

      cleanupDevBinArtifacts({ copyTo: copyRoot, outputEnvJson: envFile });

      assert.equal(fs.existsSync(copyRoot), false);
      assert.equal(fs.existsSync(envFile), false);
      assert.equal(fs.existsSync(tempRoot), true);
    } finally {
      fs.rmSync(tempRoot, { force: true, recursive: true });
    }
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

  it("reports environment setup errors as failed check results", async () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-check-setup-error-"));
    try {
      const result = await executeCheck({
        id: "missing-env-file-test",
        label: "missing env file test",
        type: PROFILE_REQUIRED,
        command: "bun",
        args: ["-e", "process.exit(0)"],
        dependsOn: [],
        envFile: path.join(tempRoot, "missing.json"),
        mutex: [],
        ignoreOutput: [],
        warningOutput: []
      });

      assert.equal(result.ok, false);
      assert.equal(result.status, "failed");
      assert.equal(result.exitCode, 1);
      assert.match(result.error?.message ?? "", /ENOENT/);
    } finally {
      fs.rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it("schedules docs validation through one executable check", () => {
    const requiredChecks = checksForProfile(PROFILE_REQUIRED);
    const docsChecks = requiredChecks.filter((check) => check.id.startsWith("docs-"));

    assert.deepEqual(docsChecks.map((check) => check.label), ["docs validators"]);
    assert.equal(reportCountForChecks(requiredChecks), 10);
  });

});

function checkByLabel(label: string) {
  const check = checks.find((candidate) => candidate.label === label);
  assert.ok(check, `expected check ${label}`);
  return check;
}

function executableName(binaryName: string): string {
  return process.platform === "win32" ? `${binaryName}.exe` : binaryName;
}
