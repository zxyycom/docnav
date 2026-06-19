import fs from "node:fs";
import path from "node:path";

import { toSlashPath } from "./path/utils.ts";

export function ensureDirForFile(filePath: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
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
    } catch {
      return;
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
