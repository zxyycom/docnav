/**
 * Baseline commit 定位与 baseline snapshot 生成。
 *
 * 从 git history 定位 previous-code baseline commit，并在临时隔离目录中
 * 用当前配置和当前 wrapper/tool 扫描 baseline commit。
 *
 * 来源：openspec/changes/implement-code-quality-observability/tasks.md tasks 3.8-3.10
 */

import { spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { randomUUID } from "node:crypto";

/**
 * 定位 previous-code baseline commit。
 *
 * 规则：
 * 1. 先确定当前配置的 scan inputs（纳入扫描的 code inputs）
 * 2. 如果 current revision 修改了任何 scan input → baseline 是 current revision 之前的最近代码提交
 * 3. 如果 current revision 没修改 scan input → baseline 是最近代码提交
 *
 * @param {object} params
 * @param {string} params.cwd - 仓库根目录
 * @param {string[]} params.scanInputPaths - 纳入扫描的文件路径模式（glob）
 * @returns {{ ok: true, sha: string, date: string, reason: string }
 *          | { ok: false, error: string }}
 */
export function locateBaselineCommit({ cwd, scanInputPaths }: any) {
  // 获取 HEAD commit
  const headResult = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (headResult.error || headResult.status !== 0) {
    return { ok: false, error: `git rev-parse HEAD failed: ${headResult.error?.message || "no git repository"}` };
  }

  const headSha = headResult.stdout.trim();

  // 尝试从 HEAD 获取最近的代码提交
  // 使用 git log 查找修改了扫描输入路径的提交
  const patternArgs = scanInputPaths.length > 0
    ? ["--", ...scanInputPaths]
    : [];

  // 查找最近一个修改了 scan inputs 的提交（不包括 HEAD，如果 HEAD 就是第一个）
  const parentCount = spawnSync("git", ["rev-list", "--count", "--max-count=1", `${headSha}^`], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  const hasParent = parentCount.status === 0 && parseInt(parentCount.stdout.trim(), 10) > 0;

  if (!hasParent) {
    // 仓库只有一个提交，没有历史
    return { ok: false, error: "no-baseline-commit: repository has only one commit" };
  }

  // 检查 HEAD 是否修改了 scan inputs
  const headDiffArgs = ["diff-tree", "--no-commit-id", "--name-only", "-r", headSha, ...patternArgs];
  const headDiff = spawnSync("git", headDiffArgs, {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  const headChangedFiles = (headDiff.stdout || "").trim().split(/\r?\n/).filter(Boolean);
  const headModifiedScanInputs = headChangedFiles.some((f) =>
    scanInputPaths.some((p: any) => fileMatchesPattern(f, p))
  );

  if (headModifiedScanInputs) {
    // HEAD 修改了 scan inputs → baseline 是 HEAD 之前的最近代码提交
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

    // 如果找不到在 scan inputs 中有变更的之前提交，使用 HEAD^
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
    // HEAD 未修改 scan inputs → baseline 是最近代码提交
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

    // 兜底：使用 HEAD^
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
 * 通过 git archive 或 worktree 导出 baseline commit 的文件，然后在隔离目录中
 * 使用当前配置和工具扫描。
 *
 * @param {object} params
 * @param {string} params.commitSha - Baseline commit SHA
 * @param {string} params.cwd - 仓库根目录
 * @param {string} params.baselineWorkDir - 隔离工作目录
 * @returns {{ ok: true, workDir: string } | { ok: false, error: string, reason: string }}
 */
export function materializeBaseline({ commitSha, cwd, baselineWorkDir }: any) {
  mkdirSync(baselineWorkDir, { recursive: true });

  // 使用 git archive 导出 baseline commit 的文件
  // git archive 输出 tar，再解压
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

  // 解压 tar
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

/**
 * 确定当前 commit 的 scan inputs 是否相对于 baseline 有变化。
 *
 * @param {object} params
 * @param {string} params.baselineSha
 * @param {string} params.cwd
 * @param {string[]} params.scanInputPaths
 * @returns {{ changed: boolean, changedFiles: string[] }}
 */
export function detectTextOnlyChange({ baselineSha, cwd, scanInputPaths }: any) {
  if (!baselineSha) {
    return { changed: true, changedFiles: [] };
  }

  const diffArgs = [
    "diff",
    "--name-only",
    `${baselineSha}..HEAD`,
    ...scanInputPaths
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
    scanInputPaths.some((p: any) => fileMatchesPattern(f, p))
  );

  return { changed: scanInputChanged, changedFiles: uniqueChangedFiles };
}

/**
 * 获取某次提交的当前更改文件列表。
 *
 * @param {string} commitSha
 * @param {string} cwd
 * @returns {string[]}
 */
export function getChangedFiles(commitSha: any, cwd: any) {
  // 对于 HEAD 提交，使用 diff-tree 获取变更文件
  const diffResult = spawnSync("git", [
    "diff-tree",
    "--no-commit-id",
    "--name-only",
    "-r",
    commitSha
  ], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });

  if (diffResult.error || diffResult.status !== 0) {
    // 尝试使用 diff with parent
    const parentResult = spawnSync("git", [
      "diff",
      "--name-only",
      `${commitSha}~1..${commitSha}`
    ], {
      cwd,
      encoding: "utf8",
      windowsHide: true
    });

    if (parentResult.status === 0) {
      return (parentResult.stdout || "").trim().split(/\r?\n/).filter(Boolean);
    }
    return [];
  }

  return (diffResult.stdout || "").trim().split(/\r?\n/).filter(Boolean);
}

// ── Helpers ───────────────────────────────────────────────────────────

function getCommitDate(sha: any, cwd: any) {
  const result = spawnSync("git", ["log", "--format=%aI", "--max-count=1", sha], {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  return (result.stdout || "").trim() || null;
}

export function getWorkingTreeChangedFiles(cwd: any, scanInputPaths: any) {
  const statusResult = spawnSync("git", [
    "status",
    "--porcelain",
    "--untracked-files=all",
    "--",
    ...scanInputPaths
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

function fileMatchesPattern(filePath: any, pattern: any) {
  // 简单的 glob 模式匹配（处理 ** 和 *）
  const regex = pattern
    .replace(/[.+^${}()|[\]\\]/g, "\\$&")
    .replace(/\*\*/g, "<<<GLOBSTAR>>>")
    .replace(/\*/g, "[^/]*")
    .replace(/<<<GLOBSTAR>>>/g, ".*");
  return new RegExp(`^${regex}$`).test(filePath);
}
