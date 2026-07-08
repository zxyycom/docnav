import type { ProcessFailure, ProcessResult } from "./types.ts";

export function processFailure(error: unknown): ProcessFailure {
  return error instanceof Error ? error as ProcessFailure : new Error(String(error));
}

export function processFailureFromResult(result: ProcessResult): ProcessFailure | null {
  if (result.error) {
    return attachProcessResult(processFailure(result.error), result);
  }
  if (result.status === 0) {
    return null;
  }
  return attachProcessResult(createExitFailure(result), result);
}

export function processFailed(result: Pick<ProcessResult, "error" | "status">): boolean {
  return result.error !== undefined || result.status !== 0;
}

function attachProcessResult(failure: ProcessFailure, result: ProcessResult): ProcessFailure {
  failure.stdout = result.stdout;
  failure.stderr = result.stderr;
  failure.signal = result.signal;
  failure.status = result.status;
  return failure;
}

function createExitFailure(result: ProcessResult): ProcessFailure {
  const failure = new Error(exitFailureMessage(result)) as ProcessFailure;
  failure.code = result.status ?? 1;
  return failure;
}

function exitFailureMessage(result: ProcessResult): string {
  if (result.status === null) {
    return `process terminated by signal ${result.signal ?? "unknown"}`;
  }
  return `process exited with status ${result.status}`;
}
