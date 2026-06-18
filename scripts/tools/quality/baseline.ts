/**
 * Baseline commit 定位与 materialization。
 *
 * 从 git history 定位 previous-code baseline commit，并在临时隔离目录中
 * 用当前配置和当前 wrapper/tool 扫描 baseline commit。
 */

import { spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { minimatch } from "minimatch";

import { gitGlobPathspecs } from "./git-pathspec.ts";

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
  const headResult = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (headResult.error || headResult.status !== 0) {
    return { ok: false, error: `git rev-parse HEAD failed: ${headResult.error?.message || "no git repository"}` };
  }

  const headSha = headResult.stdout.trim();

  const patternArgs = scanInputPaths.length > 0
    ? ["--", ...gitGlobPathspecs(scanInputPaths)]
    : [];

  const parentCount = spawnSync("git", ["rev-list", "--count", "--max-count=1", `${headSha}^`], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  const hasParent = parentCount.status === 0 && parseInt(parentCount.stdout.trim(), 10) > 0;

  if (!hasParent) {
    return { ok: false, error: "no-baseline-commit: repository has only one commit" };
  }

  const headDiffArgs = ["diff-tree", "--no-commit-id", "--name-only", "-r", headSha, ...patternArgs];
  const headDiff = spawnSync("git", headDiffArgs, {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  const headChangedFiles = (headDiff.stdout || "").trim().split(/\r?\n/).filter(Boolean);
  const headModifiedScanInputs = headChangedFiles.some((f) =>
    scanInputPaths.some((p) => fileMatchesPattern(f, p))
  );

  if (headModifiedScanInputs) {
    const logResult = spawnSync("git", [
      "log",
      "--format=%H",
      "--max-count=1",
      "--skip=0",
      `${headSha}~1`
    ].concat(patternArgs), {
      cwd,
      encoding: "utf8",
      windowsHide: true
    });

    const baselineSha = (logResult.stdout || "").trim();
    if (baselineSha) {
      return {
        ok: true,
        sha: baselineSha,
        date: getCommitDate(baselineSha, cwd),
        reason: "previous-code-commit"
      };
    }

    const parentResult = spawnSync("git", ["rev-parse", `${headSha}~1`], {
      cwd,
      encoding: "utf8",
      windowsHide: true
    });

    if (parentResult.status === 0 && parentResult.stdout.trim()) {
      const parentSha = parentResult.stdout.trim();
      return {
        ok: true,
        sha: parentSha,
        date: getCommitDate(parentSha, cwd),
        reason: "parent-commit"
      };
    }

    return { ok: false, error: "no-baseline-commit: no previous commit found" };
  } else {
    const logResult = spawnSync("git", [
      "log",
      "--format=%H",
      "--max-count=1"
    ].concat(patternArgs), {
      cwd,
      encoding: "utf8",
      windowsHide: true
    });

    const baselineSha = (logResult.stdout || "").trim();
    if (baselineSha) {
      return {
        ok: true,
        sha: baselineSha,
        date: getCommitDate(baselineSha, cwd),
        reason: "nearest-code-commit"
      };
    }

    const parentResult = spawnSync("git", ["rev-parse", `${headSha}~1`], {
      cwd,
      encoding: "utf8",
      windowsHide: true
    });

    if (parentResult.status === 0 && parentResult.stdout.trim()) {
      const parentSha = parentResult.stdout.trim();
      return {
        ok: true,
        sha: parentSha,
        date: getCommitDate(parentSha, cwd),
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

  const archiveResult = spawnSync("git", [
    "archive",
    "--format=tar",
    "--output", archivePath,
    commitSha
  ], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (archiveResult.error || archiveResult.status !== 0) {
    return {
      ok: false,
      error: `git archive failed: ${archiveResult.error?.message || archiveResult.stderr || "exit " + archiveResult.status}`,
      reason: "baseline-materialization-failed"
    };
  }

  const untarDir = join(baselineWorkDir, "repo");
  mkdirSync(untarDir, { recursive: true });

  const untarResult = spawnSync("tar", ["-xf", archivePath, "-C", untarDir], {
    cwd: baselineWorkDir,
    encoding: "utf8",
    windowsHide: true
  });

  if (untarResult.error || untarResult.status !== 0) {
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

  const diffResult = spawnSync("git", diffArgs, {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (diffResult.error) {
    return { changed: true, changedFiles: [] };
  }

  const changedFiles = [
    ...(diffResult.stdout || "").trim().split(/\r?\n/).filter(Boolean),
    ...getWorkingTreeChangedFiles(cwd, scanInputPaths)
  ].map((f) => f.replace(/\\/g, "/"));
  const uniqueChangedFiles = [...new Set(changedFiles)];
  const scanInputChanged = changedFiles.some((f) =>
    scanInputPaths.some((p) => fileMatchesPattern(f, p))
  );

  return { changed: scanInputChanged, changedFiles: uniqueChangedFiles };
}

// ── Helpers ───────────────────────────────────────────────────────────

function getCommitDate(sha: string, cwd: string): string | null {
  const result = spawnSync("git", ["log", "--format=%aI", "--max-count=1", sha], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  return (result.stdout || "").trim() || null;
}

export function getWorkingTreeChangedFiles(cwd: string, scanInputPaths: string[]): string[] {
  const statusResult = spawnSync("git", [
    "status",
    "--porcelain",
    "--untracked-files=all",
    "--",
    ...gitGlobPathspecs(scanInputPaths)
  ], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (statusResult.error || statusResult.status !== 0) {
    return [];
  }

  return (statusResult.stdout || "")
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter(Boolean)
    .map((line) => {
      const rawPath = line.slice(3).trim();
      const renameMarker = " -> ";
      return rawPath.includes(renameMarker)
        ? rawPath.slice(rawPath.indexOf(renameMarker) + renameMarker.length)
        : rawPath;
    })
    .filter(Boolean);
}

function fileMatchesPattern(filePath: string, pattern: string): boolean {
  return minimatch(filePath.replace(/\\/g, "/"), pattern);
}
