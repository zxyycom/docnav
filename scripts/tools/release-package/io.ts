import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { root } from "./config.ts";
import { ensureDirForFile } from "../fs.ts";
import { DEFAULT_PROCESS_MAX_BUFFER, processFailed, runProcessSync } from "../process.ts";
import type { ProcessResult, RunProcessSyncOptions } from "../process.ts";

export type RunCommandOptions = Pick<
  RunProcessSyncOptions,
  "encoding" | "env" | "maxBuffer" | "stdio"
> & {
  cwd?: string;
  label?: string;
};

export function runCommand(command: string, args: string[], options: RunCommandOptions = {}): ProcessResult {
  const result = runProcessSync(command, args, {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    stdio: options.stdio,
    encoding: options.encoding ?? "utf8",
    maxBuffer: options.maxBuffer ?? DEFAULT_PROCESS_MAX_BUFFER,
  });

  if (processFailed(result)) {
    const label = options.label ?? [command, ...args].join(" ");
    throw new Error(composeProcessError(label, result));
  }

  return result;
}

export function runNodeScript(
  scriptPath: string,
  args: string[] = [],
  options: RunCommandOptions = {},
): ProcessResult {
  return runCommand(process.execPath, [scriptPath, ...args], {
    ...options,
    label: `node ${path.relative(root, scriptPath)}`,
    stdio: options.stdio ?? "inherit",
  });
}

export function copyExecutable(sourcePath: string, destPath: string): void {
  ensureDirForFile(destPath);
  fs.copyFileSync(sourcePath, destPath);
  fs.chmodSync(destPath, fs.statSync(sourcePath).mode);
}

export function sha256File(filePath: string): string {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

function composeProcessError(command: string, result: ProcessResult): string {
  const details: string[] = [];
  if (result.stdout) {
    details.push(`stdout:\n${result.stdout}`);
  }
  if (result.stderr) {
    details.push(`stderr:\n${result.stderr}`);
  }
  if (result.error) {
    details.push(`process error: ${result.error.message}`);
  }

  const exitText =
    result.status === null || result.status === undefined
      ? "process-error"
      : String(result.status);
  const detailText = details.length > 0 ? `\n${details.join("\n")}` : "";
  return `${command} failed with exit ${exitText}${detailText}`;
}
