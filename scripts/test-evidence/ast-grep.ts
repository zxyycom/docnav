import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  runProcess,
  type ProcessResult
} from "../tools/foundation/src/index.ts";

export const AST_GREP_PACKAGE = "@ast-grep/cli";
export const AST_GREP_VERSION = "0.45.0";

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  ".."
);

export type RunAstGrepOptions = {
  cwd?: string;
  workspaceRoot?: string;
};

export function resolveAstGrepExecutable(root = workspaceRoot): string {
  const binaryName = process.platform === "win32" ? "ast-grep.exe" : "ast-grep";
  return path.join(root, "node_modules", AST_GREP_PACKAGE, binaryName);
}

export function runAstGrep(
  args: string[],
  options: RunAstGrepOptions = {}
): Promise<ProcessResult> {
  const root = options.workspaceRoot ?? workspaceRoot;
  const executable = resolveAstGrepExecutable(root);
  if (!fs.existsSync(executable)) {
    throw new Error(
      `missing ${AST_GREP_PACKAGE}@${AST_GREP_VERSION}; run the repository dependency bootstrap`
    );
  }
  return runProcess({
    args,
    command: executable,
    cwd: options.cwd ?? root,
    label: `ast-grep ${args.join(" ")}`
  });
}

export function expectedAstGrepVersionLine(): string {
  return `ast-grep ${AST_GREP_VERSION}`;
}
