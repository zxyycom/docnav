/**
 * Repository file discovery and fingerprint helpers for quality scans.
 */

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { createHash } from "node:crypto";
import { minimatch } from "minimatch";

import { DEFAULT_CONFIG } from "./config.ts";
import { buildFingerprint, isExcluded } from "./classify.ts";
import { getWorkingTreeChangedFiles } from "./baseline.ts";
import { gitGlobPathspecs } from "./git-pathspec.ts";
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
    "--",
    ...gitGlobPathspecs(config.include)
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
    "--",
    ...gitGlobPathspecs(config.include)
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

  const result = spawnSync("git", [
    "diff",
    "--name-only",
    "HEAD~1..HEAD",
    "--",
    ...gitGlobPathspecs(DEFAULT_CONFIG.include)
  ], {
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
        const content = normalizeFingerprintText(readFileSync(absPath, "utf8"));
        return createHash("sha256").update(content).digest("hex");
      } catch {
        return "file-not-readable";
      }
    });
  }

  return fingerprints;
}

function normalizeFingerprintText(content: string): string {
  return content.replace(/\r\n?/g, "\n");
}

function collectFilesFallback(rootDir: string, config: QualityConfig): string[] {
  const files: string[] = [];

  listFilesRecursive(rootDir, "", (relPath) => {
    if (isScanInputFile(relPath, config)) {
      files.push(relPath);
    }
  });

  return uniqueSorted(files);
}

function collectBaselineFilesFallback(workDir: string, config: QualityConfig): string[] {
  const files: string[] = [];

  listFilesRecursive(workDir, "", (relPath) => {
    if (isScanInputFile(relPath, config)) {
      files.push(relPath);
    }
  });

  return uniqueSorted(files);
}

function getChangedFilesForSingleCommitRepo(rootDir: string): string[] {
  const rootResult = spawnSync("git", [
    "diff-tree",
    "--no-commit-id",
    "--name-only",
    "-r",
    "HEAD",
    "--",
    ...gitGlobPathspecs(DEFAULT_CONFIG.include)
  ], {
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
    .filter((f) => isScanInputFile(f, config));
}

function splitGitFileList(stdout: string): string[] {
  return (stdout || "")
    .trim()
    .split(/\r?\n/)
    .filter(Boolean)
    .map((f) => f.replace(/\\/g, "/"));
}

function isScanInputFile(filePath: string, config: QualityConfig): boolean {
  const normalized = filePath.replace(/\\/g, "/");
  return config.include.some((pattern) => minimatch(normalized, pattern)) &&
    !isExcluded(normalized, config.excludeDirs, config.generatedFiles);
}

function uniqueSorted(files: string[]): string[] {
  return [...new Set(files)].sort();
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
