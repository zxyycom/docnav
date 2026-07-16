import type { ProcessResult } from "./process/types.ts";

export { processFailure, processFailureFromResult, processFailed } from "./process/failure.ts";
export { PLAIN_TEXT_PROCESS_ENV, plainTextProcessEnv } from "./process/env.ts";
export { runProcess, runProcessSync } from "./process/runner.ts";
export { DEFAULT_PROCESS_MAX_BUFFER } from "./process/types.ts";
export type {
  ProcessFailure,
  ProcessResult,
  RunProcessOptions,
  RunProcessSyncOptions
} from "./process/types.ts";

export function writeProcessOutput(result: Pick<ProcessResult, "stderr" | "stdout">): void {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}
