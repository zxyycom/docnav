import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

import {
  collectBaselineFiles,
  collectScanFiles,
  getChangedFileList,
  type ScanInputConfig
} from "./files.ts";
import {
  detectScanInputChange,
  getWorkingTreeChangedFiles,
  materializeBaselineRevision,
  type ChangeScope
} from "./revisions.ts";
import { resolveChangedInputForScan } from "../scan-command/changed-files.ts";

describe("quality changed file input", () => {
  it("fails fast when an explicit changed-files list cannot be read", () => {
    assert.throws(
      () => getChangedFileList({ changedFiles: "missing-changed-files.txt" }, process.cwd()),
      /failed to read --changed-files missing-changed-files\.txt/
    );
  });

  it("reports unavailable revision input instead of an unchanged file set", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-missing-repository-"));
    const missingRepository = join(tempDir, "missing");

    try {
      assert.throws(
        () => getWorkingTreeChangedFiles(missingRepository, []),
        /git status .* failed/
      );

      const scope = detectScanInputChange({
        baselineSha: "missing-baseline",
        cwd: missingRepository,
        scanInputPaths: ["src/**/*.ts"]
      });
      assert.equal(scope.status, "unavailable");
      if (scope.status === "unavailable") {
        assert.match(scope.reason, /git diff .* failed/);
      }

      const repository = join(tempDir, "repository");
      initializeRepository(repository);
      writeFixtureFile(repository, "src/only.ts", "export const only = true;\n");
      commitAll(repository, "only commit");

      const invalidBaselineScope = detectScanInputChange({
        baselineSha: "missing-baseline",
        cwd: repository,
        scanInputPaths: ["absent/**/*.ts"]
      });
      assert.equal(invalidBaselineScope.status, "unavailable");
      if (invalidBaselineScope.status === "unavailable") {
        assert.match(invalidBaselineScope.reason, /git diff .* failed/);
      }
      assert.deepEqual(
        getChangedFileList({ scanInputPaths: ["src/**/*.ts"] }, repository),
        ["src/only.ts"]
      );

      const explicit = resolveChangedInputForScan({
        opts: { changedFiles: "caller-owned.txt" },
        root: repository,
        scope: invalidBaselineScope,
        collectChangedFiles: () => ["src/only.ts"]
      });
      assert.deepEqual(explicit, {
        changedFiles: ["src/only.ts"],
        inputScope: {
          status: "available",
          changed: true,
          changedFiles: ["src/only.ts"]
        }
      });

      const detectedChange = {
        status: "available",
        changed: true,
        changedFiles: ["src/only.ts"]
      } satisfies ChangeScope;
      assert.deepEqual(resolveChangedInputForScan({
        opts: { changedFiles: "caller-owned-empty.txt" },
        root: repository,
        scope: detectedChange,
        collectChangedFiles: () => []
      }), {
        changedFiles: [],
        inputScope: detectedChange
      });
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it("keeps current, changed, and baseline repository files aligned", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-repository-"));
    const repository = join(tempDir, "repository");
    const committedPath = "src/committed.ts";
    const untrackedPath = "src/untracked.ts";
    const workingPath = "src/working.ts";
    const config = {
      excludeDirs: [".git"],
      generatedFiles: [],
      include: ["src/**/*.ts"]
    } satisfies ScanInputConfig;

    try {
      initializeRepository(repository);
      writeFixtureFile(repository, committedPath, "export const committed = 1;\n");
      writeFixtureFile(repository, workingPath, "export const working = 1;\n");
      const baselineSha = commitAll(repository, "baseline");
      writeFixtureFile(repository, committedPath, "export const committed = 2;\n");
      commitAll(repository, "current");
      writeFixtureFile(repository, workingPath, "export const working = 2;\n");
      writeFixtureFile(repository, untrackedPath, "export const untracked = true;\n");

      assert.deepEqual(
        collectScanFiles(repository, config),
        [committedPath, untrackedPath, workingPath]
      );

      const scope = detectScanInputChange({
        baselineSha,
        cwd: repository,
        scanInputPaths: config.include
      });
      if (scope.status !== "available") assert.fail(scope.reason);
      assert.equal(scope.changed, true);
      assert.deepEqual(scope.changedFiles.sort(), [committedPath, untrackedPath, workingPath]);
      assert.deepEqual(
        getChangedFileList({ scanInputPaths: config.include }, repository).sort(),
        [committedPath, untrackedPath, workingPath]
      );

      const materialized = materializeBaselineRevision({
        baselineWorkDir: join(tempDir, "materialized"),
        commitSha: baselineSha,
        cwd: repository
      });
      assert.equal(materialized.ok, true, materialized.ok ? undefined : materialized.error);
      if (!materialized.ok) return;

      assert.deepEqual(collectBaselineFiles(materialized.workDir, config), [committedPath, workingPath]);
      assert.equal(
        readFileSync(join(materialized.workDir, committedPath), "utf8").trim(),
        "export const committed = 1;"
      );
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

function writeFixtureFile(rootDir: string, relPath: string, content: string): void {
  const absPath = join(rootDir, relPath);
  mkdirSync(dirname(absPath), { recursive: true });
  writeFileSync(absPath, content, "utf8");
}

function initializeRepository(repository: string): void {
  mkdirSync(repository, { recursive: true });
  git(repository, ["init", "--quiet"]);
  git(repository, ["config", "user.email", "quality-test@example.invalid"]);
  git(repository, ["config", "user.name", "Quality Test"]);
}

function commitAll(repository: string, message: string): string {
  git(repository, ["add", "."]);
  git(repository, ["commit", "--quiet", "-m", message]);
  return git(repository, ["rev-parse", "HEAD"]);
}

function git(repository: string, args: string[]): string {
  const result = spawnSync("git", args, {
    cwd: repository,
    encoding: "utf8"
  });
  assert.equal(
    result.status,
    0,
    `git ${args.join(" ")} failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`
  );
  return result.stdout.trim();
}
