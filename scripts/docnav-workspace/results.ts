import type { ProcessFailure } from "../tools/process.ts";
import type { CheckReportRef, CheckStatus, CheckTask } from "./checks/index.ts";
import { reportIdForCheck, reportLabelForCheck, visibleOutputLines } from "./checks/index.ts";

export interface CompletionResult {
  check: CheckReportRef;
  combinedOutput: string;
  durationMs: number;
  endedAtMs: number;
  error: ProcessFailure | null;
  exitCode: number;
  ok: boolean;
  startedAtMs: number;
  status: CheckStatus;
  stderr: string;
  stdout: string;
  visibleOutput: string;
}

export interface CheckResult extends CompletionResult {
  check: CheckTask;
}

interface ReportAccumulator {
  check: CheckReportRef;
  completed: number;
  endedAtMs: number;
  error: ProcessFailure | null;
  exitCode: number;
  expected: number;
  ok: boolean;
  startedAtMs: number;
  status: CheckStatus;
  visibleOutputChunks: string[];
}

export function formatCompletionLine(result: Pick<CompletionResult, "check" | "durationMs" | "status">): string {
  return `  ${result.status}: ${result.check.label} (${formatDurationMs(result.durationMs)})`;
}

export function formatDurationMs(durationMs: number): string {
  if (!Number.isFinite(durationMs)) {
    return "unknown";
  }
  if (durationMs < 1000) {
    return `${Math.max(0, Math.round(durationMs))}ms`;
  }
  const totalSeconds = durationMs / 1000;
  if (totalSeconds < 60) {
    return `${totalSeconds.toFixed(totalSeconds < 10 ? 1 : 0)}s`;
  }
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = Math.round(totalSeconds % 60).toString().padStart(2, "0");
  return `${minutes}m ${seconds}s`;
}

export function createReportCompletionTracker(checkList: readonly CheckTask[]): (result: CheckResult) => CompletionResult | null {
  const reports = createReportAccumulators(checkList);

  return (result: CheckResult) => {
    const report = reports.get(reportIdForCheck(result.check));
    if (!report) {
      throw new Error(`missing report for check: ${result.check.id}`);
    }
    recordReportCompletion(report, result);
    if (report.completed !== report.expected) {
      return null;
    }
    return completeReportResult(report);
  };
}

export function visibleOutputForCheck(check: CheckTask, output: string, status: CheckStatus = "failed"): string {
  return visibleOutputLines(check, output, status).join("\n");
}

function createReportAccumulators(checkList: readonly CheckTask[]): Map<string, ReportAccumulator> {
  const reports = new Map<string, ReportAccumulator>();
  for (const check of checkList) {
    const reportId = reportIdForCheck(check);
    const report = reports.get(reportId) ?? createReportAccumulator(check, reportId);
    report.expected += 1;
    reports.set(reportId, report);
  }
  return reports;
}

function createReportAccumulator(check: CheckTask, reportId: string): ReportAccumulator {
  return {
    check: {
      id: reportId,
      label: reportLabelForCheck(check)
    },
    expected: 0,
    completed: 0,
    ok: true,
    exitCode: 0,
    error: null,
    startedAtMs: Number.POSITIVE_INFINITY,
    endedAtMs: 0,
    status: "passed",
    visibleOutputChunks: []
  };
}

function recordReportCompletion(report: ReportAccumulator, result: CheckResult): void {
  report.completed += 1;
  report.ok &&= result.ok;
  report.startedAtMs = Math.min(report.startedAtMs, result.startedAtMs);
  report.endedAtMs = Math.max(report.endedAtMs, result.endedAtMs);
  report.status = combineStatus(report.status, result.status);
  if (result.visibleOutput.length > 0) {
    report.visibleOutputChunks.push(result.visibleOutput);
  }
  if (!result.ok && !report.error) {
    report.error = result.error;
    report.exitCode = result.exitCode;
  }
}

function completeReportResult(report: ReportAccumulator): CompletionResult {
  const visibleOutput = report.visibleOutputChunks.join("\n");
  return {
    check: report.check,
    ok: report.ok,
    exitCode: report.exitCode,
    error: report.error,
    status: report.status,
    stdout: "",
    stderr: "",
    combinedOutput: visibleOutput,
    visibleOutput,
    durationMs: report.endedAtMs - report.startedAtMs,
    startedAtMs: report.startedAtMs,
    endedAtMs: report.endedAtMs
  };
}

function combineStatus(current: CheckStatus, next: CheckStatus): CheckStatus {
  if (current === "failed" || next === "failed") {
    return "failed";
  }
  if (current === "warning" || next === "warning") {
    return "warning";
  }
  return "passed";
}
