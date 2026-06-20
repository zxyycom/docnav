import fs from "node:fs";

import { walk } from "../repo/files.ts";
import { toAbs, toRel } from "../repo/paths.ts";

export function readText(relPath: string): string {
  return fs.readFileSync(toAbs(relPath), "utf8");
}

export function sortedUnique(values: string[]): string[] {
  return [...new Set(values)].sort();
}

export function listMainMarkdownDocs(): string[] {
  const docs = walk(toAbs("docs"), (filePath) => {
    const relPath = toRel(filePath);
    return (
      relPath.endsWith(".md") &&
      !relPath.startsWith("docs/examples/") &&
      !relPath.startsWith("docs/schemas/")
    );
  }).map(toRel);

  return sortedUnique(["README.md", "AGENTS.md", ...docs]);
}
