import fs from "node:fs";
import path from "node:path";

import { FILE_SYSTEM } from "./config.mjs";
import { root, toRel, walk } from "./fs-utils.mjs";

export function validateMarkdownLinks() {
  const markdownFiles = walk(root, (filePath) =>
    filePath.endsWith(FILE_SYSTEM.markdownExtension)
  );
  const missing = [];
  const linkPattern = /\[[^\]]+\]\(([^)]+)\)/g;

  for (const filePath of markdownFiles) {
    const text = fs.readFileSync(filePath, "utf8");
    for (const match of text.matchAll(linkPattern)) {
      const rawTarget = match[1].trim().replace(/^<|>$/g, "");
      if (
        rawTarget === "" ||
        rawTarget.startsWith("#") ||
        /^(https?|mailto):/i.test(rawTarget)
      ) {
        continue;
      }

      const targetPath = rawTarget.split("#")[0];
      if (targetPath === "") {
        continue;
      }

      const resolved = path.resolve(path.dirname(filePath), targetPath);
      if (!fs.existsSync(resolved)) {
        missing.push(`${toRel(filePath)} -> ${rawTarget}`);
      }
    }
  }

  if (missing.length > 0) {
    throw new Error(`missing markdown links:\n${missing.join("\n")}`);
  }

  console.log(`markdown links ok: ${markdownFiles.length} file(s)`);
}
