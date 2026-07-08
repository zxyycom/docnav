import { runParallelTasks } from "../../tools/parallel-task-runner/src/index.ts";
import {
  asCheckTask,
  checksForProfile,
  reportCountForChecks
} from "../checks/index.ts";
import type { CheckResult, CompletionResult } from "../results.ts";
import { createReportCompletionTracker } from "../results.ts";
import type { VerificationOptions } from "./args.ts";
import { executeCheck } from "./execution.ts";
import { appendLog, createLogPaths, finalizeLogs, initializeLogs } from "./logs.ts";
import { printCompletionResult, printHeader, printSummary } from "./output.ts";

export async function runVerification({ profile, concurrency }: VerificationOptions): Promise<number> {
  const selectedChecks = checksForProfile(profile);
  const totalReports = reportCountForChecks(selectedChecks);
  const completeReport = createReportCompletionTracker(selectedChecks);
  const logPaths = createLogPaths();
  initializeLogs(logPaths, profile, totalReports, selectedChecks.length);

  const startedAtMs = Date.now();
  const completedReports: CompletionResult[] = [];

  printHeader(profile, totalReports);
  await runParallelTasks<CheckResult>(selectedChecks, {
    concurrency,
    execute: (task) => executeCheck(asCheckTask(task)),
    onComplete: (result) => {
      appendLog(logPaths, result);
      const report = completeReport(result);
      if (report) {
        completedReports.push(report);
        printCompletionResult(report);
      }
    }
  });

  const failures = completedReports.filter((result) => !result.ok);
  const totalDurationMs = Date.now() - startedAtMs;
  finalizeLogs(logPaths, totalDurationMs);

  printSummary({
    profile,
    totalChecks: totalReports,
    completedResults: completedReports,
    totalDurationMs,
    logPaths
  });

  return failures.length > 0 ? 1 : 0;
}
