/**
 * Repository file discovery and fingerprint helpers for quality scans.
 */

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { createHash } from "node:crypto";

import { DEFAULT_CONFIG } from "./config.ts";
import { buildFingerprint, isExcluded } from "./classify.ts";
import { getWorkingTreeChangedFiles } from "./baseline.ts";
import type { CodeAreaFileMap, CodeAreaFingerprint, QualityConfig } from "./schema.ts";

export type ChangedFilesOptions = {
  changedFiles?: string | null;
};

export function collectScanFiles(rootDir: string, config: QualityConfig): string[] {
  const result = spawnSync("git", [
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    ...config.include
  ], {
    cwd: rootDir,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });

  if (result.error || result.status !== 0) {
    console.log("  ⚠️  git ls-files failed, using fallback file collection");
    return collectFilesFallback(rootDir, config);
  }

  const allFiles = (result.stdout || "").trim().split(/\r?\n/).filter(Boolean);

  return normalizeAndFilterFiles(allFiles, config, rootDir);
}

export function collectBaselineFiles(workDir: string, config: QualityConfig): string[] {
  const result = spawnSync("git", [
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    ...config.include
  ], {
    cwd: workDir,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });

  if (result.status === 0 && result.stdout.trim()) {
    return normalizeAndFilterFiles((result.stdout || "").trim().split(/\r?\n/).filter(Boolean), config, workDir);
  }

  return collectBaselineFilesFallback(workDir, config);
}

export function getChangedFileList(opts: ChangedFilesOptions, rootDir: string): string[] {
  if (opts.changedFiles) {
    try {
      return readFileSync(opts.changedFiles, "utf8")
        .split(/\r?\n/)
        .filter(Boolean)
        .map((f) => f.replace(/\\/g, "/"));
    } catch {
      return [];
    }
  }

  const result = spawnSync("git", ["diff", "--name-only", "HEAD~1..HEAD"], {
    cwd: rootDir,
    encoding: "utf8",
    windowsHide: true
  });

  if (result.status !== 0) {
    return getChangedFilesForSingleCommitRepo(rootDir);
  }

  const committedChangedFiles = splitGitFileList(result.stdout);
  const workingTreeChangedFiles = getWorkingTreeChangedFiles(rootDir, DEFAULT_CONFIG.include)
    .map((f) => f.replace(/\\/g, "/"));

  return [...new Set([...committedChangedFiles, ...workingTreeChangedFiles])];
}

export function buildFingerprints(fileMap: CodeAreaFileMap, rootDir: string): Record<string, CodeAreaFingerprint> {
  const fingerprints: Record<string, CodeAreaFingerprint> = {};

  for (const [area, files] of fileMap.entries()) {
    fingerprints[area] = buildFingerprint(area, files, (filePath) => {
      const absPath = resolve(rootDir, filePath);
      try {
        const content = readFileSync(absPath, "utf8");
        return createHash("sha256").update(content).digest("hex");
      } catch {
        return "file-not-readable";
      }
    });
  }

  return fingerprints;
}

function collectFilesFallback(rootDir: string, config: QualityConfig): string[] {
  const files: string[] = [];

  for (const pattern of config.include) {
    const result = spawnSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "--", pattern], {
      cwd: rootDir,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: 1024 * 1024 * 64
    });

    if (result.status === 0) {
      files.push(...splitGitFileList(result.stdout));
    }
  }

  return files.filter((f) => !isExcluded(f, config.excludeDirs, config.generatedFiles));
}

function collectBaselineFilesFallback(workDir: string, config: QualityConfig): string[] {
  const files: string[] = [];

  for (const pattern of config.include) {
    const globPattern = pattern
      .replace(/\*\*/g, "<<<GLOBSTAR>>>")
      .replace(/\*/g, "[^/]*")
      .replace(/<<<GLOBSTAR>>>/g, ".*");

    const fileRegex = new RegExp(`^${globPattern}$`);
    listFilesRecursive(workDir, "", (relPath) => {
      if (fileRegex.test(relPath) && !isExcluded(relPath, config.excludeDirs, config.generatedFiles)) {
        files.push(relPath);
      }
    });
  }

  return [...new Set(files)].sort();
}

function getChangedFilesForSingleCommitRepo(rootDir: string): string[] {
  const rootResult = spawnSync("git", ["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"], {
    cwd: rootDir,
    encoding: "utf8",
    windowsHide: true
  });

  const workingTreeChangedFiles = getWorkingTreeChangedFiles(rootDir, DEFAULT_CONFIG.include)
    .map((f) => f.replace(/\\/g, "/"));

  if (rootResult.status === 0) {
    return [...new Set([...splitGitFileList(rootResult.stdout), ...workingTreeChangedFiles])];
  }

  return [...new Set(workingTreeChangedFiles)];
}

function normalizeAndFilterFiles(files: string[], config: QualityConfig, rootDir: string): string[] {
  return files
    .map((f) => f.replace(/\\/g, "/"))
    .filter((f) => existsSync(resolve(rootDir, f)))
    .filter((f) => !isExcluded(f, config.excludeDirs, config.generatedFiles));
}

function splitGitFileList(stdout: string): string[] {
  return (stdout || "")
    .trim()
    .split(/\r?\n/)
    .filter(Boolean)
    .map((f) => f.replace(/\\/g, "/"));
}

function listFilesRecursive(baseDir: string, subDir: string, callback: (relPath: string) => void): void {
  const currentDir = subDir ? join(baseDir, subDir) : baseDir;
  let entries;

  try {
    entries = readdirSync(currentDir, { withFileTypes: true });
  } catch {
    return;
  }

  for (const entry of entries) {
    const relPath = subDir ? `${subDir}/${entry.name}` : entry.name;

    if (entry.isDirectory()) {
      listFilesRecursive(baseDir, relPath, callback);
    } else if (entry.isFile()) {
      callback(relPath);
    }
  }
}
