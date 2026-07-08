import path from "node:path";

import { FILE_SYSTEM } from "../config.ts";
import { walkFiles } from "../../foundation/src/fs.ts";

const ignoredDirs = new Set(FILE_SYSTEM.ignoredDirs);

export function walk(dir: string, predicate: (filePath: string) => boolean = () => true): string[] {
  return walkFiles(dir, { ignoredDirs })
    .map((relPath) => path.join(dir, relPath))
    .filter(predicate);
}
