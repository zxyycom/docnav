import fs from "node:fs";
import path from "node:path";

import { toSlashPath } from "./path.ts";

export function ensureDirForFile(filePath: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

export function assertStrictDescendantPath(
  parentPath: string,
  candidatePath: string,
  candidateLabel: string,
  parentLabel: string
): void {
  const relative = path.relative(path.resolve(parentPath), path.resolve(candidatePath));
  if (
    relative.length === 0
    || path.isAbsolute(relative)
    || relative === ".."
    || relative.startsWith(`..${path.sep}`)
  ) {
    throw new Error(`${candidateLabel} must be a strict child of the ${parentLabel}`);
  }
}

export function assertNoSymlinkPathSegments(
  rootDir: string,
  candidatePath: string,
  label: string
): void {
  const root = path.resolve(rootDir);
  const candidate = path.resolve(candidatePath);
  assertStrictDescendantPath(root, candidate, label, `root ${root}`);
  const relative = path.relative(root, candidate);

  let current = root;
  for (const segment of relative.split(path.sep)) {
    current = path.join(current, segment);
    let stat: fs.Stats;
    try {
      stat = fs.lstatSync(current);
    } catch (error) {
      if (isMissingPathError(error)) return;
      throw new Error(`failed to inspect ${label} path segment ${current}`, { cause: error });
    }
    if (stat.isSymbolicLink()) {
      throw new Error(`${label} contains symbolic link path segment ${current}`);
    }
  }
}

export function readTextFile(filePath: string): string {
  return fs.readFileSync(filePath, "utf8");
}

export function writeTextFile(filePath: string, content: string): void {
  ensureDirForFile(filePath);
  fs.writeFileSync(filePath, content, "utf8");
}

export function readJsonFile(filePath: string): unknown {
  return JSON.parse(readTextFile(filePath));
}

export function writeJsonFile(
  filePath: string,
  value: unknown,
  options: { trailingNewline?: boolean } = {}
): void {
  const content = JSON.stringify(value, null, 2);
  writeTextFile(filePath, options.trailingNewline === false ? content : `${content}\n`);
}

export function walkFiles(
  rootDir: string,
  options: { ignoredDirs?: Iterable<string> } = {}
): string[] {
  const ignoredDirs = new Set(options.ignoredDirs ?? []);
  const results: string[] = [];

  const visit = (subDir: string) => {
    const currentDir = subDir ? path.join(rootDir, subDir) : rootDir;
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(currentDir, { withFileTypes: true });
    } catch (error) {
      throw new Error(`failed to read directory ${currentDir}`, { cause: error });
    }

    for (const entry of entries) {
      const relPath = subDir ? `${subDir}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        if (!ignoredDirs.has(entry.name)) {
          visit(relPath);
        }
      } else if (entry.isFile()) {
        results.push(toSlashPath(relPath));
      }
    }
  };

  visit("");
  return results;
}

function isMissingPathError(error: unknown): boolean {
  return error instanceof Error
    && "code" in error
    && error.code === "ENOENT";
}
