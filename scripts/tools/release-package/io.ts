import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import type { SpawnSyncOptionsWithStringEncoding, SpawnSyncReturns } from "node:child_process";

import { root } from "./config.ts";

// Cargo JSON 构建输出可能较大，统一保留 64 MiB 缓冲以避免结果被截断。
const DEFAULT_MAX_BUFFER = 1024 * 1024 * 64;

export type RunCommandOptions = Omit<
  SpawnSyncOptionsWithStringEncoding,
  "cwd" | "encoding" | "env" | "maxBuffer" | "windowsHide"
> & {
  cwd?: string;
  encoding?: BufferEncoding;
  env?: NodeJS.ProcessEnv;
  label?: string;
  maxBuffer?: number;
};

export function runCommand(command: string, args: string[], options: RunCommandOptions = {}): SpawnSyncReturns<string> {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    stdio: options.stdio,
    encoding: options.encoding ?? "utf8",
    windowsHide: true,
    maxBuffer: options.maxBuffer ?? DEFAULT_MAX_BUFFER,
  });

  if (result.error || result.status !== 0) {
    const label = options.label ?? [command, ...args].join(" ");
    throw new Error(composeSpawnError(label, result));
  }

  return result;
}

export function runNodeScript(
  scriptPath: string,
  args: string[] = [],
  options: RunCommandOptions = {},
): SpawnSyncReturns<string> {
  return runCommand(process.execPath, [scriptPath, ...args], {
    ...options,
    label: `node ${path.relative(root, scriptPath)}`,
    stdio: options.stdio ?? "inherit",
  });
}

export function copyExecutable(sourcePath: string, destPath: string): void {
  fs.mkdirSync(path.dirname(destPath), { recursive: true });
  fs.copyFileSync(sourcePath, destPath);
  fs.chmodSync(destPath, fs.statSync(sourcePath).mode);
}

export function readJsonFile(filePath: string): unknown {
  const parsed: unknown = JSON.parse(fs.readFileSync(filePath, "utf8"));
  return parsed;
}

export function writeJsonFile(filePath: string, value: unknown): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

export function writeTextFile(filePath: string, content: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
}

export function readTextFile(filePath: string): string {
  return fs.readFileSync(filePath, "utf8");
}

export function sha256File(filePath: string): string {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

export function normalizeRelativePath(filePath: string): string {
  return filePath.replaceAll(path.sep, "/");
}

function composeSpawnError(command: string, result: SpawnSyncReturns<string>): string {
  const details: string[] = [];
  if (result.stdout) {
    details.push(`stdout:\n${result.stdout}`);
  }
  if (result.stderr) {
    details.push(`stderr:\n${result.stderr}`);
  }
  if (result.error) {
    details.push(`spawn error: ${result.error.message}`);
  }

  const exitText =
    result.status === null || result.status === undefined
      ? "spawn-error"
      : String(result.status);
  const detailText = details.length > 0 ? `\n${details.join("\n")}` : "";
  return `${command} failed with exit ${exitText}${detailText}`;
}
