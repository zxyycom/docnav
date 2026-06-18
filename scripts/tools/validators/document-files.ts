import fs from "node:fs";

import { toAbs, toRel, walk } from "./fs-utils.ts";

export function readText(relPath: ExternalValue) {
  return fs.readFileSync(toAbs(relPath), "utf8");
}

export function sortedUnique(values: ExternalValue) {
  return [...new Set(values)].sort();
}

export function listMainMarkdownDocs() {
  const docs = walk(toAbs("docs"), (filePath: ExternalValue) => {
    const relPath = toRel(filePath);
    return (
      relPath.endsWith(".md") &&
      !relPath.startsWith("docs/examples/") &&
      !relPath.startsWith("docs/schemas/")
    );
  }).map(toRel);

  return sortedUnique(["README.md", "AGENTS.md", ...docs]);
}
