import { processFailure, processFailureFromResult, runProcess } from "../../tools/foundation/src/process.ts";
import type { ProcessFailure } from "../../tools/foundation/src/process.ts";
import type { CheckStatus } from "../checks/index.ts";
import type { CheckTask } from "../checks/index.ts";
import type { CheckResult } from "../results.ts";
import { visibleOutputForCheck } from "../results.ts";
import { environmentForCheck } from "./environment.ts";
import { root } from "./paths.ts";

const MAX_BUFFER = 1024 * 1024 * 64;

interface CheckExecutionData {
  endedAtMs: number;
  error?: unknown;
  exitCode: number;
  ok: boolean;
  startedAtMs: number;
  status: CheckStatus;
  stderr: string;
  stdout: string;
}

export async function executeCheck(check: CheckTask): Promise<CheckResult> {
  const startedAtMs = Date.now();
  try {
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
      status: statusForExecution(check, failure === null, result.stdout, result.stderr),
      startedAtMs,
      endedAtMs: Date.now()
    });
  } catch (error: unknown) {
    const failure = processFailure(error);
    return buildCheckResult(check, {
      ok: false,
      exitCode: normalizeExitCode(failure),
      stdout: "",
      stderr: failure.message,
      error: failure,
      status: "failed",
      startedAtMs,
      endedAtMs: Date.now()
    });
  }
}

function buildCheckResult(check: CheckTask, data: CheckExecutionData): CheckResult {
  const combinedOutput = combinedProcessOutput(data);
  return {
    check,
    ok: data.ok,
    exitCode: data.exitCode,
    error: data.error === undefined ? null : processFailure(data.error),
    status: data.status,
    stdout: data.stdout,
    stderr: data.stderr,
    combinedOutput,
    visibleOutput: visibleOutputForCheck(check, combinedOutput, data.status),
    durationMs: data.endedAtMs - data.startedAtMs,
    startedAtMs: data.startedAtMs,
    endedAtMs: data.endedAtMs
  };
}

function statusForExecution(
  check: CheckTask,
  ok: boolean,
  stdout: string,
  stderr: string
): CheckStatus {
  if (!ok) {
    return "failed";
  }
  const output = combinedProcessOutput({ stdout, stderr });
  return check.warningOutput.some((pattern) => pattern.test(output)) ? "warning" : "passed";
}

function combinedProcessOutput({ stdout, stderr }: Pick<CheckExecutionData, "stderr" | "stdout">): string {
  return [stdout, stderr].filter(Boolean).join("\n");
}

function normalizeExitCode(error: ProcessFailure): number {
  return typeof error?.code === "number" ? error.code : 1;
}
