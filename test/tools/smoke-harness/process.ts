import { Buffer } from "node:buffer";
import { spawn } from "node:child_process";

import { createCommandOutputCapture } from "./process/output.ts";

export const MAX_COMMAND_OUTPUT = 1024 * 1024 * 64;

export interface SmokeCommandOptions {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  maxBuffer?: number;
  project?: { env?: NodeJS.ProcessEnv; root?: string };
  stdin?: Buffer | string | null;
  stdinSummary?: string | null;
}

export interface ProcessResult {
  error: string | null;
  exitCode: number;
  signal: NodeJS.Signals | null;
  stderr: string;
  stdout: string;
}

export interface PreparedCliCommand {
  cwd: string;
  executable: string;
  processOptions: SmokeCommandOptions;
}

export function createProcessOptions(
  commandOptions: SmokeCommandOptions,
  cwd: string,
  env: NodeJS.ProcessEnv | undefined
): SmokeCommandOptions {
  return {
    cwd,
    env,
    stdin: commandOptions.stdin,
    maxBuffer: MAX_COMMAND_OUTPUT
  };
}

export function normalizeProcessResult(result: ProcessResult): ProcessResult {
  return {
    exitCode: defaultValue(result.exitCode, 1),
    signal: defaultValue(result.signal, null),
    error: defaultValue(result.error, null),
    stdout: defaultValue(result.stdout, ""),
    stderr: defaultValue(result.stderr, "")
  };
}

export function summarizeCommandStdin(commandOptions: SmokeCommandOptions): string | null {
  return commandOptions.stdinSummary ?? summarizeStdin(commandOptions.stdin);
}

export function spawnCommand(executable: string, args: string[], options: SmokeCommandOptions): Promise<ProcessResult> {
  return new Promise((resolve) => {
    let childError: Error | null = null;
    let settled = false;
    const maxBuffer = options.maxBuffer ?? MAX_COMMAND_OUTPUT;

    const child = spawn(executable, args, {
      cwd: options.cwd,
      env: options.env,
      windowsHide: true,
      stdio: "pipe"
    });

    const output = createCommandOutputCapture(maxBuffer, () => {
      if (!child.killed) {
        childError = new Error(`command output exceeded ${maxBuffer} bytes`);
        child.kill();
      }
    });

    child.stdout.on("data", (chunk: unknown) => output.append(chunk, "stdout"));
    child.stderr.on("data", (chunk: unknown) => output.append(chunk, "stderr"));
    child.on("error", (error: Error) => {
      childError = error;
      finish(1, null);
    });
    child.on("close", (exitCode, signal) => finish(exitCode, signal));
    child.stdin.on("error", () => {});

    if (options.stdin !== undefined && options.stdin !== null) {
      child.stdin.end(options.stdin);
    } else {
      child.stdin.end();
    }

    function finish(exitCode: number | null, signal: NodeJS.Signals | null) {
      if (settled) {
        return;
      }
      settled = true;
      const { stdout, stderr } = output.snapshot();
      resolve({
        exitCode: exitCode ?? 1,
        signal,
        error: childError?.message ?? null,
        stdout,
        stderr
      });
    }
  });
}

function defaultValue<T>(value: T | null | undefined, fallback: T): T {
  return value ?? fallback;
}

function summarizeStdin(stdin: Buffer | string | null | undefined): string | null {
  if (stdin === undefined || stdin === null) {
    return null;
  }
  const byteCount = Buffer.isBuffer(stdin) ? stdin.length : Buffer.byteLength(String(stdin), "utf8");
  const unit = byteCount === 1 ? "byte" : "bytes";
  return `${byteCount} ${unit} stdin`;
}
