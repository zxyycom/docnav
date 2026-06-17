import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

import { root } from "./config.ts";

// Cargo JSON 构建输出可能较大，统一保留 64 MiB 缓冲以避免结果被截断。
const DEFAULT_MAX_BUFFER = 1024 * 1024 * 64;

export function runCommand(command: any, args: any, options: any = {}) {
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

export function runNodeScript(scriptPath: any, args: any[] = [], options: any = {}) {
  return runCommand(process.execPath, [scriptPath, ...args], {
    ...options,
    label: `node ${path.relative(root, scriptPath)}`,
    stdio: options.stdio ?? "inherit",
  });
}

export function copyExecutable(sourcePath: any, destPath: any) {
  fs.mkdirSync(path.dirname(destPath), { recursive: true });
  fs.copyFileSync(sourcePath, destPath);
  fs.chmodSync(destPath, fs.statSync(sourcePath).mode);
}

export function readJsonFile(filePath: any) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

export function writeJsonFile(filePath: any, value: any) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

export function writeTextFile(filePath: any, content: any) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
}

export function readTextFile(filePath: any) {
  return fs.readFileSync(filePath, "utf8");
}

export function sha256File(filePath: any) {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

export function normalizeRelativePath(filePath: any) {
  return filePath.replaceAll(path.sep, "/");
}

function composeSpawnError(command: any, result: any) {
  const details: any[] = [];
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
