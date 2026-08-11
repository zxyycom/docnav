import { describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  cleanupDevBinArtifacts,
  prepareDevBinEnv
} from "../docnav-dev/build-bins.ts";
import {
  DEV_BIN_COPY_DIR,
  DEV_BIN_ENV_FILE,
  resolveDevBinArtifactPaths
} from "../docnav-dev/artifacts.ts";
import {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  checks,
  checksForProfile,
  reportCountForChecks,
  visibleOutputLines
} from "./checks/index.ts";
import { formatCompletionLine, formatDurationMs } from "./results.ts";
import { parseArgs, resolveVerificationConcurrency } from "./verify/args.ts";
import { executeCheck } from "./verify/execution.ts";
import { printCompletionResult } from "./verify/output.ts";

describe("workspace verifier configuration", () => {
  it("filters known success noise from terminal-visible output", () => {
    const cases = [
      {
        checkId: "test-evidence-ledger",
        output: [
          "$ bun scripts/test-evidence/index.ts check --root .",
          "Test Case check passed: 537 current test entities (393 Cargo, 117 Bun, 27 smoke); 537 mapped by 116 semantic Cases across 11 topics."
        ].join("\n")
      },
      {
        checkId: "typecheck-scripts",
        output: "$ tsgo -p tsconfig.json"
      },
      {
        checkId: "lint-scripts",
        output: "$ eslint --max-warnings 0 --cache --cache-location .eslintcache --cache-strategy content"
      }
    ];

    for (const { checkId, output } of cases) {
      assert.deepEqual(visibleOutputLines(checkById(checkId), output), []);
    }
  });

  it("keeps actionable failure output after filtering known success noise", () => {
    const cases = [
      {
        checkId: "test-evidence-ledger",
        output: [
          "$ bun scripts/test-evidence/index.ts check --root .",
          "unexpected diagnostic"
        ].join("\n"),
        expected: ["unexpected diagnostic"]
      },
      {
        checkId: "docs-validators",
        output: [
          "Decision records check passed (1 domains, 2 decisions, 1 active, 1 aligned, 0 unaligned, 1 archived, 0 candidates).",
          "schema diagnostic"
        ].join("\n"),
        expected: ["schema diagnostic"]
      }
    ];

    for (const { checkId, output, expected } of cases) {
      assert.deepEqual(visibleOutputLines(checkById(checkId), output, "failed"), expected);
    }
  });

  it("suppresses all passed output even when a success line is not configured", () => {
    const check = checkById("docs-validators");
    const output = [
      "protocol examples ok: 4 operation(s)",
      "unexpected diagnostic"
    ].join("\n");

    assert.deepEqual(visibleOutputLines(check, output, "passed"), []);
  });

  it("filters quality timing details from terminal-visible output", () => {
    const check = checkById("quality-full-check");
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
    const check = checkById("cargo-test");
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

  it("prints visible warning output immediately after completion lines", () => {
    const lines: string[] = [];

    printCompletionResult(
      {
        status: "warning",
        check: { id: "docs-validators", label: "docs validators" },
        durationMs: 1250,
        visibleOutput: "catalog diagnostic\nschema diagnostic"
      },
      (line) => lines.push(line)
    );

    assert.deepEqual(lines, [
      "  warning: docs validators (1.3s)",
      "catalog diagnostic\nschema diagnostic"
    ]);
  });

  it("separates required and full verification profiles", () => {
    const requiredIds = checksForProfile(PROFILE_REQUIRED).map((check) => check.id);
    const fullIds = checksForProfile(PROFILE_FULL).map((check) => check.id);

    for (const id of [
      "typecheck-scripts",
      "lint-scripts",
      "change-plans",
      "docs-validators",
      "test-evidence-ledger",
      "quality-quick-check"
    ]) {
      assert.ok(requiredIds.includes(id), `required profile should include ${id}`);
    }
    for (const id of [
      "typecheck-scripts",
      "test-evidence-ledger",
      "quality-full-check",
      "cargo-test",
      "docnav-core-development-smoke",
      "change-plans"
    ]) {
      assert.ok(fullIds.includes(id), `full profile should include ${id}`);
    }
    assert.ok(!requiredIds.includes("quality-full-check"));
    assert.ok(!fullIds.includes("quality-quick-check"));
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
      fs.mkdirSync(sourceDir, { recursive: true });

      const docnavSource = path.join(sourceDir, executableName("docnav"));
      fs.writeFileSync(docnavSource, "docnav");

      const env = prepareDevBinEnv({
        docnavExecutable: docnavSource,
        workspaceRoot: tempRoot
      });

      assert.notEqual(env.DOCNAV_BIN, docnavSource);
      assert.match(
        path.relative(path.join(tempRoot, DEV_BIN_COPY_DIR), env.DOCNAV_BIN),
        /^run-[^\\/]+[\\/]docnav(?:\.exe)?$/
      );
      assert.equal(fs.readFileSync(env.DOCNAV_BIN, "utf8"), "docnav");
    } finally {
      fs.rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it("removes copied development binary artifacts", () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-dev-bins-cleanup-"));
    try {
      const { copyRoot, envFile } = resolveDevBinArtifactPaths(tempRoot);
      fs.mkdirSync(path.join(copyRoot, "run-example"), { recursive: true });
      fs.writeFileSync(envFile, "{}");

      cleanupDevBinArtifacts(tempRoot);

      assert.equal(fs.existsSync(copyRoot), false);
      assert.equal(fs.existsSync(envFile), false);
      assert.equal(fs.existsSync(tempRoot), true);
    } finally {
      fs.rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it("keeps development binary cleanup paths owned by the dev-bin script", () => {
    const buildCheck = checkById("docnav-development-binaries");
    const cleanupCheck = checkById("docnav-development-artifacts-cleanup");

    assert.deepEqual(buildCheck.args, [
      "scripts/docnav-dev/build-bins.ts",
      "--quiet"
    ]);
    assert.deepEqual(cleanupCheck.args, [
      "scripts/docnav-dev/build-bins.ts",
      "--cleanup"
    ]);

    const managedRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-managed-paths-"));
    try {
      const paths = resolveDevBinArtifactPaths(managedRoot);
      assert.equal(
        path.relative(paths.artifactRoot, paths.copyRoot),
        path.relative(".cache/docnav/verify", DEV_BIN_COPY_DIR)
      );
      assert.equal(
        path.relative(paths.artifactRoot, paths.envFile),
        path.relative(".cache/docnav/verify", DEV_BIN_ENV_FILE)
      );
    } finally {
      fs.rmSync(managedRoot, { force: true, recursive: true });
    }

    if (process.platform !== "win32") {
      const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-dev-bin-symlink-"));
      const outsideRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-dev-bin-outside-"));
      try {
        fs.symlinkSync(outsideRoot, path.join(tempRoot, ".cache"), "dir");
        assert.throws(
          () => resolveDevBinArtifactPaths(tempRoot),
          /contains symbolic link path segment/
        );
      } finally {
        fs.rmSync(tempRoot, { force: true, recursive: true });
        fs.rmSync(outsideRoot, { force: true, recursive: true });
      }
    }
  });

  it("formats completion lines and durations for streaming output", () => {
    assert.equal(formatDurationMs(234), "234ms");
    assert.equal(formatDurationMs(1250), "1.3s");
    assert.equal(formatDurationMs(65_000), "1m 05s");
    assert.equal(formatDurationMs(59_800), "1m 00s");
    assert.equal(formatDurationMs(119_800), "2m 00s");
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

    assert.deepEqual(docsChecks.map((check) => check.id), ["docs-validators"]);
    assert.equal(reportCountForChecks(requiredChecks), 9);
  });
});

function checkById(id: string) {
  const check = checks.find((candidate) => candidate.id === id);
  assert.ok(check, `expected check ${id}`);
  return check;
}

function executableName(binaryName: string): string {
  return process.platform === "win32" ? `${binaryName}.exe` : binaryName;
}
