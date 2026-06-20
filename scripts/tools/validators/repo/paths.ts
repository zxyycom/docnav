import path from "node:path";
import { fileURLToPath } from "node:url";

import { toSlashPath } from "../../path.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");

export function toAbs(relPath: string): string {
  return path.join(root, relPath);
}

export function toRel(absPath: string): string {
  return toSlashPath(path.relative(root, absPath));
}
