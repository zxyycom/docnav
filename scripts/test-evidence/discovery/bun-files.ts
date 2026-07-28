import fs from "node:fs";
import path from "node:path";

import { minimatch } from "minimatch";

import {
  isSafeRelativePosixPath,
  resolveExistingWorkspacePath
} from "../relative-path.ts";

const GLOB_OPTIONS = {
  dot: true,
  nocomment: true,
  nonegate: true
} as const;

export type BunTestSurface = {
  sourceRoots: string[];
  include: string[];
  ignore: string[];
  supplementalFiles: string[];
};

export function resolveBunTestFiles(options: {
  workspaceRoot: string;
  profile: BunTestSurface;
}): string[] {
  validateSurface(options.profile);

  const includeMatches = new Map(
    options.profile.include.map((pattern) => [pattern, 0])
  );
  const directoryFiles = collectDirectoryFiles(options, includeMatches);
  assertEveryIncludeMatched(includeMatches);
  const files = new Set(directoryFiles);
  addSupplementalFiles(options, directoryFiles, files);

  if (files.size === 0) {
    throw new Error("Bun test surface matched no files");
  }
  return [...files].sort(compareStrings);
}

function collectDirectoryFiles(
  options: {
    workspaceRoot: string;
    profile: BunTestSurface;
  },
  includeMatches: Map<string, number>
): Set<string> {
  const files = new Set<string>();
  for (const sourceRoot of options.profile.sourceRoots) {
    const resolved = resolveExistingWorkspacePath(
      options.workspaceRoot,
      sourceRoot,
      `Bun source root ${sourceRoot}`
    );
    if (!resolved.stats.isDirectory()) {
      throw new Error(`Bun source root must be a directory: ${sourceRoot}`);
    }
    collectRootFiles(
      options.profile,
      sourceRoot,
      resolved.absolutePath,
      includeMatches,
      files
    );
  }
  return files;
}

function collectRootFiles(
  profile: BunTestSurface,
  sourceRoot: string,
  rootPath: string,
  includeMatches: Map<string, number>,
  files: Set<string>
): void {
  for (const relativePath of walkRegularFiles(rootPath)) {
    const matchedPatterns = profile.include.filter((pattern) => (
      minimatch(relativePath, pattern, GLOB_OPTIONS)
    ));
    for (const pattern of matchedPatterns) {
      includeMatches.set(pattern, (includeMatches.get(pattern) ?? 0) + 1);
    }
    const ignored = profile.ignore.some((pattern) => (
      minimatch(relativePath, pattern, GLOB_OPTIONS)
    ));
    if (matchedPatterns.length > 0 && !ignored) {
      files.add(path.posix.join(sourceRoot, relativePath));
    }
  }
}

function assertEveryIncludeMatched(includeMatches: Map<string, number>): void {
  for (const [pattern, matchCount] of includeMatches) {
    if (matchCount === 0) {
      throw new Error(`Bun include pattern matched no files: ${pattern}`);
    }
  }
}

function addSupplementalFiles(
  options: {
    workspaceRoot: string;
    profile: BunTestSurface;
  },
  directoryFiles: ReadonlySet<string>,
  files: Set<string>
): void {
  for (const sourcePath of options.profile.supplementalFiles) {
    const resolved = resolveExistingWorkspacePath(
      options.workspaceRoot,
      sourcePath,
      `Bun supplemental file ${sourcePath}`
    );
    if (!resolved.stats.isFile()) {
      throw new Error(`Bun supplemental file must be a file: ${sourcePath}`);
    }
    if (directoryFiles.has(sourcePath)) {
      throw new Error(
        `Bun supplemental file is already included by directory rules: ${sourcePath}`
      );
    }
    files.add(sourcePath);
  }
}

function validateSurface(profile: BunTestSurface): void {
  if (
    profile.sourceRoots.length === 0 ||
    profile.include.length === 0
  ) {
    throw new Error("Bun sourceRoots and include must be non-empty");
  }
  for (const sourceRoot of profile.sourceRoots) {
    if (!isSafeRelativePosixPath(sourceRoot)) {
      throw new Error(
        `Bun sourceRoots must contain safe relative POSIX paths: ${sourceRoot}`
      );
    }
  }
  for (const pattern of [...profile.include, ...profile.ignore]) {
    if (!isSafeRelativeGlob(pattern)) {
      throw new Error(
        `Bun patterns must be positive relative POSIX globs: ${pattern}`
      );
    }
  }
  for (const sourcePath of profile.supplementalFiles) {
    if (!isSafeRelativePosixPath(sourcePath)) {
      throw new Error(
        `Bun supplementalFiles must contain safe relative POSIX paths: ${sourcePath}`
      );
    }
  }
}

function walkRegularFiles(rootPath: string): string[] {
  const files: string[] = [];
  visit(rootPath, "");
  return files;

  function visit(directoryPath: string, relativeDirectory: string): void {
    const entries = fs.readdirSync(directoryPath, { withFileTypes: true })
      .sort((left, right) => compareStrings(left.name, right.name));
    for (const entry of entries) {
      const relativePath = relativeDirectory
        ? path.posix.join(relativeDirectory, entry.name)
        : entry.name;
      const absolutePath = path.join(directoryPath, entry.name);
      if (entry.isDirectory()) {
        visit(absolutePath, relativePath);
      } else if (entry.isFile()) {
        files.push(relativePath);
      }
    }
  }
}

export function isSafeRelativeGlob(pattern: string): boolean {
  return (
    pattern.length > 0 &&
    pattern === pattern.trim() &&
    !pattern.startsWith("!") &&
    !pattern.startsWith("#") &&
    !pattern.includes("\\") &&
    !pattern.includes("\0") &&
    !path.posix.isAbsolute(pattern) &&
    pattern.split("/").every((segment) => (
      segment.length > 0 &&
      segment !== "." &&
      segment !== ".."
    ))
  );
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
