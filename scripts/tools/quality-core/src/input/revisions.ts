/**
 * Baseline commit 定位与 materialization。
 *
 * 从 git history 定位 previous-code baseline commit，并在临时隔离目录中
 * 用当前配置和当前 wrapper/tool 扫描 baseline commit。
 */

import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { minimatch } from "minimatch";

import { gitGlobPathspecArgs } from "./git-pathspec.ts";
import {
  errorMessage,
  gitCommitDate,
  gitHeadSha,
  parseGitStatusPaths,
  processFailed,
  runGit,
  runProcessSync,
  splitGitFileList,
  toSlashPath,
  type ProcessResult
} from "../../../foundation/src/index.ts";

type BaselineCommitResult =
  | { date: string | null; ok: true; reason: string; sha: string }
  | { error: string; ok: false };

type MaterializeBaselineResult =
  | { ok: true; workDir: string }
  | { error: string; ok: false; reason: string };

export type ChangeScope =
  | {
      status: "available";
      changed: boolean;
      changedFiles: string[];
    }
  | {
      status: "unavailable";
      reason: string;
    };

/**
 * 定位 previous-code baseline commit。
 *
 * 规则：
 * 1. 先确定当前配置的 scan inputs（纳入扫描的 code inputs）
 * 2. 如果 current revision 修改了任何 scan input → baseline 是 current revision 之前的最近代码提交
 * 3. 如果 current revision 没修改 scan input → baseline 是最近代码提交
 */
export function locateBaselineCommit({
  cwd,
  scanInputPaths
}: {
  cwd: string;
  scanInputPaths: string[];
}): BaselineCommitResult {
  const headSha = gitHeadSha(cwd);
  if (!headSha) {
    return { ok: false, error: "git rev-parse HEAD failed: no git repository" };
  }

  if (!hasParentCommit(cwd, headSha)) {
    return { ok: false, error: "no-baseline-commit: repository has only one commit" };
  }

  const patternArgs = gitGlobPathspecArgs(scanInputPaths, { omitWhenEmpty: true });

  let headModifiedScanInputs: boolean;
  try {
    headModifiedScanInputs = commitModifiesScanInputs({
      cwd,
      headSha,
      scanInputPaths
    });
  } catch (error) {
    return { ok: false, error: errorMessage(error) };
  }

  if (headModifiedScanInputs) {
    return baselineForChangedHead(cwd, headSha, patternArgs);
  }

  return baselineForUnchangedHead(cwd, headSha, patternArgs);
}

/**
 * 在隔离目录中生成 baseline snapshot。
 *
 * 通过 git archive 导出文件；导出的目录不是 git repo。
 */
export function materializeBaselineRevision({
  commitSha,
  cwd,
  baselineWorkDir
}: {
  baselineWorkDir: string;
  commitSha: string;
  cwd: string;
}): MaterializeBaselineResult {
  mkdirSync(baselineWorkDir, { recursive: true });

  const archivePath = join(baselineWorkDir, "baseline.tar");

  const archiveResult = runGit([
    "archive",
    "--format=tar",
    "--output", archivePath,
    commitSha
  ], {
    cwd
  });

  if (processFailed(archiveResult)) {
    return {
      ok: false,
      error: `git archive failed: ${archiveResult.error?.message || archiveResult.stderr || "exit " + archiveResult.status}`,
      reason: "baseline-materialization-failed"
    };
  }

  const untarDir = join(baselineWorkDir, "repo");
  mkdirSync(untarDir, { recursive: true });

  const untarResult = runProcessSync("tar", ["-xf", archivePath, "-C", untarDir], { cwd: baselineWorkDir });

  if (processFailed(untarResult)) {
    return {
      ok: false,
      error: `tar extract failed: ${untarResult.error?.message || untarResult.stderr || "exit " + untarResult.status}`,
      reason: "baseline-materialization-failed"
    };
  }

  return { ok: true, workDir: untarDir };
}

export function detectScanInputChange({
  baselineSha,
  cwd,
  scanInputPaths
}: {
  baselineSha: string | null;
  cwd: string;
  scanInputPaths: string[];
}): ChangeScope {
  try {
    const changedFiles = [
      ...(baselineSha
        ? getRevisionChangedFiles(cwd, baselineSha, "HEAD", scanInputPaths)
        : []),
      ...getWorkingTreeChangedFiles(cwd, scanInputPaths)
    ].map(toSlashPath);
    const uniqueChangedFiles = uniqueSortedPaths(changedFiles);
    const scanInputChanged = baselineSha
      ? changedFiles.some((file) =>
          scanInputPaths.length === 0
          || scanInputPaths.some((pattern) => fileMatchesPattern(file, pattern))
        )
      : true;

    return {
      status: "available",
      changed: scanInputChanged,
      changedFiles: uniqueChangedFiles
    };
  } catch (error) {
    return {
      status: "unavailable",
      reason: errorMessage(error)
    };
  }
}

// ── Helpers ───────────────────────────────────────────────────────────

export function getWorkingTreeChangedFiles(cwd: string, scanInputPaths: string[]): string[] {
  const result = runGit(["status", "--porcelain", "--untracked-files=all"], {
    cwd,
    maxBuffer: 1024 * 1024 * 64
  });
  if (processFailed(result)) {
    throw gitCommandError("git status --porcelain --untracked-files=all", cwd, result);
  }
  return filterScanInputFiles(parseGitStatusPaths(result.stdout), scanInputPaths);
}

export function getRevisionChangedFiles(
  cwd: string,
  fromRevision: string,
  toRevision: string,
  scanInputPaths: string[] | readonly string[]
): string[] {
  const result = runGit(["diff", "--name-only", `${fromRevision}..${toRevision}`], {
    cwd,
    maxBuffer: 1024 * 1024 * 64
  });
  if (processFailed(result)) {
    throw gitCommandError(
      `git diff --name-only ${fromRevision}..${toRevision}`,
      cwd,
      result
    );
  }
  return filterScanInputFiles(splitGitFileList(result.stdout), scanInputPaths);
}

/**
 * Collect a useful file set for a one-commit repository where HEAD~1 does not
 * exist. Unlike change detection, this conservative UI input does not claim
 * that a revision diff was successfully observed.
 */
export function getRevisionChangedFilesOrRevisionSnapshot(
  cwd: string,
  fromRevision: string,
  toRevision: string,
  scanInputPaths: string[] | readonly string[]
): string[] {
  try {
    return getRevisionChangedFiles(cwd, fromRevision, toRevision, scanInputPaths);
  } catch (diffError) {
    const fromRevisionExists = runGit(
      ["rev-parse", "--verify", "--quiet", `${fromRevision}^{commit}`],
      { cwd }
    ).status === 0;
    if (fromRevisionExists) throw diffError;

    try {
      return filterScanInputFiles(filesAtRevision(cwd, toRevision), scanInputPaths);
    } catch (fallbackError) {
      throw new Error(`${errorMessage(diffError)}; fallback failed: ${errorMessage(fallbackError)}`, {
        cause: fallbackError
      });
    }
  }
}

function filesAtRevision(cwd: string, revision: string): string[] {
  const result = runGit(["ls-tree", "-r", "--name-only", revision], {
    cwd,
    maxBuffer: 1024 * 1024 * 64
  });
  if (processFailed(result)) {
    throw gitCommandError(`git ls-tree -r --name-only ${revision}`, cwd, result);
  }
  return splitGitFileList(result.stdout);
}

function gitCommandError(
  command: string,
  cwd: string,
  result: Pick<ProcessResult, "error" | "status" | "stderr">
): Error {
  const detail = result.error?.message || result.stderr.trim() || `exit ${result.status}`;
  return new Error(`${command} failed in ${cwd}: ${detail}`, {
    cause: result.error ?? undefined
  });
}

function filterScanInputFiles(
  files: readonly string[],
  scanInputPaths: readonly string[]
): string[] {
  return uniqueSortedPaths(
    files
      .map(toSlashPath)
      .filter((file) => scanInputPaths.length === 0
        || scanInputPaths.some((pattern) => fileMatchesPattern(file, pattern)))
  );
}

function uniqueSortedPaths(files: readonly string[]): string[] {
  return [...new Set(files)].sort();
}

function hasParentCommit(cwd: string, headSha: string): boolean {
  const parentCount = runGit(["rev-list", "--count", "--max-count=1", `${headSha}^`], { cwd });
  return parentCount.status === 0 && parseInt(parentCount.stdout.trim(), 10) > 0;
}

function baselineForChangedHead(cwd: string, headSha: string, patternArgs: string[]): BaselineCommitResult {
  const baselineSha = latestCodeCommitBeforeHead(cwd, headSha, patternArgs);
  if (baselineSha) {
    return baselineCommit(cwd, baselineSha, "previous-code-commit");
  }

  const parentBaseline = parentBaselineCommit(cwd, headSha, "parent-commit");
  if (parentBaseline) {
    return parentBaseline;
  }

  return { ok: false, error: "no-baseline-commit: no previous commit found" };
}

function baselineForUnchangedHead(cwd: string, headSha: string, patternArgs: string[]): BaselineCommitResult {
  const baselineSha = latestCodeCommit(cwd, patternArgs);
  if (baselineSha) {
    return baselineCommit(cwd, baselineSha, "nearest-code-commit");
  }

  const parentBaseline = parentBaselineCommit(cwd, headSha, "parent-commit-fallback");
  if (parentBaseline) {
    return parentBaseline;
  }

  return { ok: false, error: "no-baseline-commit: no previous code commit found" };
}

function commitModifiesScanInputs({
  cwd,
  headSha,
  scanInputPaths
}: {
  cwd: string;
  headSha: string;
  scanInputPaths: string[];
}): boolean {
  const parent = `${headSha}^`;
  return getRevisionChangedFiles(cwd, parent, headSha, scanInputPaths).length > 0;
}

function latestCodeCommitBeforeHead(cwd: string, headSha: string, patternArgs: string[]): string | null {
  return latestCommit(cwd, [
    "log",
    "--format=%H",
    "--max-count=1",
    "--skip=0",
    `${headSha}~1`,
    ...patternArgs
  ]);
}

function latestCodeCommit(cwd: string, patternArgs: string[]): string | null {
  return latestCommit(cwd, ["log", "--format=%H", "--max-count=1", ...patternArgs]);
}

function latestCommit(cwd: string, args: string[]): string | null {
  const logResult = runGit(args, { cwd });
  return trimStdout(logResult.stdout);
}

function parentBaselineCommit(cwd: string, headSha: string, reason: string): BaselineCommitResult | null {
  const parentResult = runGit(["rev-parse", `${headSha}~1`], { cwd });
  const parentSha = parentResult.status === 0 ? trimStdout(parentResult.stdout) : null;
  return parentSha ? baselineCommit(cwd, parentSha, reason) : null;
}

function baselineCommit(cwd: string, sha: string, reason: string): BaselineCommitResult {
  return {
    ok: true,
    sha,
    date: gitCommitDate(sha, cwd),
    reason
  };
}

function trimStdout(stdout: string | null | undefined): string | null {
  const value = (stdout || "").trim();
  return value || null;
}

function fileMatchesPattern(filePath: string, pattern: string): boolean {
  return minimatch(toSlashPath(filePath), pattern);
}
