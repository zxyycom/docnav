import { processFailure, processFailureFromResult, runProcess } from "../../tools/process.ts";
import type { ProcessFailure } from "../../tools/process.ts";
import type { CheckTask } from "../checks/index.ts";
import type { CheckResult } from "../results.ts";
import { environmentForCheck } from "./environment.ts";
import { root } from "./paths.ts";

const MAX_BUFFER = 1024 * 1024 * 64;

interface CheckExecutionData {
  endedAtMs: number;
  error?: unknown;
  exitCode: number;
  ok: boolean;
  startedAtMs: number;
  stderr: string;
  stdout: string;
}

export async function executeCheck(check: CheckTask): Promise<CheckResult> {
  const startedAtMs = Date.now();
  const result = await runProcess({
    args: check.args,
    command: check.command,
    cwd: root,
    env: environmentForCheck(check),
    maxBuffer: MAX_BUFFER
  });
  const failure = processFailureFromResult(result);
  return buildCheckResult(check, {
    ok: failure === null,
    exitCode: failure === null ? 0 : normalizeExitCode(failure),
    stdout: result.stdout,
    stderr: result.stderr,
    error: failure ?? undefined,
    startedAtMs,
    endedAtMs: Date.now()
  });
}

function buildCheckResult(check: CheckTask, data: CheckExecutionData): CheckResult {
  return {
    check,
    ok: data.ok,
    exitCode: data.exitCode,
    error: data.error === undefined ? null : processFailure(data.error),
    stdout: data.stdout,
    stderr: data.stderr,
    combinedOutput: combinedProcessOutput(data),
    durationMs: data.endedAtMs - data.startedAtMs,
    startedAtMs: data.startedAtMs,
    endedAtMs: data.endedAtMs
  };
}

function combinedProcessOutput({ stdout, stderr }: Pick<CheckExecutionData, "stderr" | "stdout">): string {
  return [stdout, stderr].filter(Boolean).join("\n");
}

function normalizeExitCode(error: ProcessFailure): number {
  return typeof error?.code === "number" ? error.code : 1;
}
