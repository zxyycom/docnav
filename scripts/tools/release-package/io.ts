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
  const result = runProcessSync(command, args, resolveRunCommandOptions(options));
  assertCommandSucceeded(command, args, options, result);
  return result;
}

function resolveRunCommandOptions(options: RunCommandOptions): RunProcessSyncOptions {
  return {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    stdio: options.stdio,
    encoding: options.encoding ?? "utf8",
    maxBuffer: options.maxBuffer ?? DEFAULT_PROCESS_MAX_BUFFER,
  };
}

function assertCommandSucceeded(
  command: string,
  args: string[],
  options: RunCommandOptions,
  result: ProcessResult,
): void {
  if (!processFailed(result)) {
    return;
  }
  throw new Error(
    composeProcessError(commandLabel(command, args, options), result),
  );
}

function commandLabel(
  command: string,
  args: string[],
  options: RunCommandOptions,
): string {
  return options.label ?? [command, ...args].join(" ");
}

export function runScript(
  scriptPath: string,
  args: string[] = [],
  options: RunCommandOptions = {},
): ProcessResult {
  return runCommand("bun", [scriptPath, ...args], {
    ...options,
    label: `bun ${path.relative(root, scriptPath)}`,
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
