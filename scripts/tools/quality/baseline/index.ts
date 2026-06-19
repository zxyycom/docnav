/**
 * Baseline commit 定位与 materialization。
 *
 * 从 git history 定位 previous-code baseline commit，并在临时隔离目录中
 * 用当前配置和当前 wrapper/tool 扫描 baseline commit。
 */

import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { minimatch } from "minimatch";

import { gitGlobPathspecs } from "../git/pathspec.ts";
import { gitCommitDate, gitHeadSha, parseGitStatusPaths, runGit, splitGitFileList } from "../../git.ts";
import { processFailed, runProcessSync } from "../../process.ts";
import { toSlashPath } from "../../path/utils.ts";

type BaselineCommitResult =
  | { date: string | null; ok: true; reason: string; sha: string }
  | { error: string; ok: false };

type MaterializeBaselineResult =
  | { ok: true; workDir: string }
  | { error: string; ok: false; reason: string };

type ChangeScope = {
  changed: boolean;
  changedFiles: string[];
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

  const patternArgs = scanInputPaths.length > 0
    ? ["--", ...gitGlobPathspecs(scanInputPaths)]
    : [];

  const parentCount = runGit(["rev-list", "--count", "--max-count=1", `${headSha}^`], { cwd });

  const hasParent = parentCount.status === 0 && parseInt(parentCount.stdout.trim(), 10) > 0;

  if (!hasParent) {
    return { ok: false, error: "no-baseline-commit: repository has only one commit" };
  }

  const headDiffArgs = ["diff-tree", "--no-commit-id", "--name-only", "-r", headSha, ...patternArgs];
  const headDiff = runGit(headDiffArgs, { cwd });

  const headChangedFiles = splitGitFileList(headDiff.stdout);
  const headModifiedScanInputs = headChangedFiles.some((f) =>
    scanInputPaths.some((p) => fileMatchesPattern(f, p))
  );

  if (headModifiedScanInputs) {
    const logResult = runGit([
      "log",
      "--format=%H",
      "--max-count=1",
      "--skip=0",
      `${headSha}~1`
    ].concat(patternArgs), { cwd });

    const baselineSha = (logResult.stdout || "").trim();
    if (baselineSha) {
      return {
        ok: true,
        sha: baselineSha,
        date: gitCommitDate(baselineSha, cwd),
        reason: "previous-code-commit"
      };
    }

    const parentResult = runGit(["rev-parse", `${headSha}~1`], { cwd });

    if (parentResult.status === 0 && parentResult.stdout.trim()) {
      const parentSha = parentResult.stdout.trim();
      return {
        ok: true,
        sha: parentSha,
        date: gitCommitDate(parentSha, cwd),
        reason: "parent-commit"
      };
    }

    return { ok: false, error: "no-baseline-commit: no previous commit found" };
  } else {
    const logResult = runGit([
      "log",
      "--format=%H",
      "--max-count=1"
    ].concat(patternArgs), { cwd });

    const baselineSha = (logResult.stdout || "").trim();
    if (baselineSha) {
      return {
        ok: true,
        sha: baselineSha,
        date: gitCommitDate(baselineSha, cwd),
        reason: "nearest-code-commit"
      };
    }

    const parentResult = runGit(["rev-parse", `${headSha}~1`], { cwd });

    if (parentResult.status === 0 && parentResult.stdout.trim()) {
      const parentSha = parentResult.stdout.trim();
      return {
        ok: true,
        sha: parentSha,
        date: gitCommitDate(parentSha, cwd),
        reason: "parent-commit-fallback"
      };
    }

    return { ok: false, error: "no-baseline-commit: no previous code commit found" };
  }
}

/**
 * 在隔离目录中生成 baseline snapshot。
 *
 * 通过 git archive 导出文件；导出的目录不是 git repo。
 */
export function materializeBaseline({
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

export function detectTextOnlyChange({
  baselineSha,
  cwd,
  scanInputPaths
}: {
  baselineSha: string | null;
  cwd: string;
  scanInputPaths: string[];
}): ChangeScope {
  if (!baselineSha) {
    return { changed: true, changedFiles: [] };
  }

  const diffArgs = [
    "diff",
    "--name-only",
    `${baselineSha}..HEAD`,
    "--",
    ...gitGlobPathspecs(scanInputPaths)
  ];

  const diffResult = runGit(diffArgs, { cwd });

  if (diffResult.error) {
    return { changed: true, changedFiles: [] };
  }

  const changedFiles = [
    ...splitGitFileList(diffResult.stdout),
    ...getWorkingTreeChangedFiles(cwd, scanInputPaths)
  ].map(toSlashPath);
  const uniqueChangedFiles = [...new Set(changedFiles)];
  const scanInputChanged = changedFiles.some((f) =>
    scanInputPaths.some((p) => fileMatchesPattern(f, p))
  );

  return { changed: scanInputChanged, changedFiles: uniqueChangedFiles };
}

// ── Helpers ───────────────────────────────────────────────────────────

export function getWorkingTreeChangedFiles(cwd: string, scanInputPaths: string[]): string[] {
  const statusResult = runGit([
    "status",
    "--porcelain",
    "--untracked-files=all",
    "--",
    ...gitGlobPathspecs(scanInputPaths)
  ], {
    cwd
  });

  if (processFailed(statusResult)) {
    return [];
  }

  return parseGitStatusPaths(statusResult.stdout);
}

function fileMatchesPattern(filePath: string, pattern: string): boolean {
  return minimatch(toSlashPath(filePath), pattern);
}
