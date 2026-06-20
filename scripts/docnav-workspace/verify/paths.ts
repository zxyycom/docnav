import path from "node:path";
import { fileURLToPath } from "node:url";

import { toSlashPath } from "../../tools/path.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const logDir = path.join(root, ".log", "verify-docnav-workspace");

export function resolveWorkspacePath(workspacePath: string): string {
  return path.resolve(root, workspacePath);
}

export function relativeLogPath(logPath: string): string {
  return toSlashPath(path.relative(root, logPath));
}
