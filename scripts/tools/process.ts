import { execa, execaSync } from "execa";
import type { SyncOptions } from "execa";

import { processFailure } from "./types.ts";
import type { ProcessFailure } from "./types.ts";

export const DEFAULT_PROCESS_MAX_BUFFER = 1024 * 1024 * 64;

export type ProcessResult = {
  error?: Error;
  signal: NodeJS.Signals | null;
  status: number | null;
  stderr: string;
  stdout: string;
};

export type RunProcessSyncOptions = {
  cwd?: string | URL;
  encoding?: SyncOptions["encoding"];
  env?: NodeJS.ProcessEnv;
  maxBuffer?: number;
  stdio?: SyncOptions["stdio"];
  timeout?: number;
  windowsHide?: boolean;
};

export type RunProcessOptions = {
  args?: string[];
  command: string;
  cwd?: string | URL;
  env?: NodeJS.ProcessEnv;
  label?: string;
  maxBuffer?: number;
  timeout?: number;
  windowsHide?: boolean;
};

export function runProcessSync(
  command: string,
  args: string[],
  options: RunProcessSyncOptions = {}
): ProcessResult {
  const result = execaSync(command, args, {
    ...options,
    encoding: options.encoding ?? "utf8",
    maxBuffer: options.maxBuffer ?? DEFAULT_PROCESS_MAX_BUFFER,
    reject: false,
    stripFinalNewline: false,
    windowsHide: options.windowsHide ?? true
  });
  return toProcessResult(result, command);
}

export function runProcess({
  args = [],
  command,
  cwd,
  env,
  label = command,
  maxBuffer = DEFAULT_PROCESS_MAX_BUFFER,
  timeout,
  windowsHide = true
}: RunProcessOptions): Promise<ProcessResult> {
  return execa(command, args, {
    cwd,
    env,
    maxBuffer,
    reject: false,
    stripFinalNewline: false,
    timeout,
    windowsHide
  }).then((result) => toProcessResult(result, label));
}

export function processFailureFromResult(result: ProcessResult): ProcessFailure | null {
  if (result.error) {
    const failure = processFailure(result.error);
    failure.stdout = result.stdout;
    failure.stderr = result.stderr;
    failure.signal = result.signal;
    failure.status = result.status;
    return failure;
  }
  if (result.status === 0) {
    return null;
  }
  const failure = new Error(
    result.status === null
      ? `process terminated by signal ${result.signal ?? "unknown"}`
      : `process exited with status ${result.status}`
  ) as ProcessFailure;
  failure.code = result.status ?? 1;
  failure.status = result.status;
  failure.signal = result.signal;
  failure.stdout = result.stdout;
  failure.stderr = result.stderr;
  return failure;
}

export function processFailed(result: Pick<ProcessResult, "error" | "status">): boolean {
  return result.error !== undefined || result.status !== 0;
}

export function writeProcessOutput(result: Pick<ProcessResult, "stderr" | "stdout">): void {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

type ExecaResultLike = {
  code?: string;
  exitCode?: number;
  failed?: boolean;
  isMaxBuffer?: boolean;
  message?: string;
  originalMessage?: string;
  shortMessage?: string;
  signal?: NodeJS.Signals;
  stderr?: unknown;
  stdout?: unknown;
  timedOut?: boolean;
};

function toProcessResult(result: ExecaResultLike, label: string): ProcessResult {
  return {
    error: processErrorFor(result, label),
    signal: result.signal ?? null,
    status: result.exitCode ?? null,
    stderr: outputString(result.stderr),
    stdout: outputString(result.stdout)
  };
}

function outputString(output: unknown): string {
  return typeof output === "string" ? output : "";
}

function processErrorFor(result: ExecaResultLike, label = "process"): Error | undefined {
  if (!isExecutionError(result)) {
    return undefined;
  }

  const message = result.originalMessage ?? result.shortMessage ?? result.message ?? `${label} failed`;
  const error = new Error(message) as NodeJS.ErrnoException;
  if (result.code) {
    error.code = result.code;
  }
  return error;
}

function isExecutionError(result: ExecaResultLike): boolean {
  if (!result.failed) {
    return false;
  }
  if (typeof result.exitCode === "number") {
    return false;
  }
  return Boolean(result.code || result.timedOut || result.isMaxBuffer || result.signal);
}
