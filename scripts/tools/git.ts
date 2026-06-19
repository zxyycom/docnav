import { processFailed, runProcessSync } from "./process.ts";
import type { ProcessResult, RunProcessSyncOptions } from "./process.ts";
import { toSlashPath } from "./path/utils.ts";

export function runGit(args: string[], options: RunProcessSyncOptions = {}): ProcessResult {
  return runProcessSync("git", args, options);
}

export function gitHeadSha(cwd: string): string | null {
  const result = runGit(["rev-parse", "HEAD"], { cwd });
  if (processFailed(result)) return null;
  return result.stdout.trim() || null;
}

export function gitCommitDate(sha: string, cwd: string): string | null {
  return gitLogField("%aI", sha, cwd);
}

export function gitCommitTitle(sha: string, cwd: string): string | null {
  return gitLogField("%s", sha, cwd);
}

export function splitGitFileList(stdout: string): string[] {
  return stdout
    .trim()
    .split(/\r?\n/)
    .filter(Boolean)
    .map(toSlashPath);
}

export function parseGitStatusPaths(stdout: string): string[] {
  return stdout
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
    .filter(Boolean)
    .map(toSlashPath);
}

function gitLogField(format: string, sha: string, cwd: string): string | null {
  const result = runGit(["log", `--format=${format}`, "--max-count=1", sha], { cwd });
  if (processFailed(result)) return null;
  return result.stdout.trim() || null;
}
