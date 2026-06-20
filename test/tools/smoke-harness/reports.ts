import type { NormalizedTask } from "../../../scripts/tools/parallel-task-runner/index.ts";

export interface SmokeTestResult {
  commandCount: number;
  durationMs: number;
  endedAtMs: number;
  error?: Error;
  id: string;
  label: string;
  ok: boolean;
  reportId?: string;
  reportLabel?: string;
  reportOrder?: number;
  startedAtMs: number;
}

export function withSmokeTaskMetadata(result: SmokeTestResult, task: NormalizedTask): SmokeTestResult {
  return {
    ...result,
    reportId: task.reportId as string | undefined,
    reportLabel: task.reportLabel as string | undefined,
    reportOrder: task.reportOrder as number | undefined
  };
}

export function formatTestResult(result: SmokeTestResult) {
  const status = result.ok ? "PASS" : "FAIL";
  const error = result.error ? `: ${result.error.message}` : "";
  const duration = Number.isFinite(result.durationMs) ? `, ${result.durationMs}ms` : "";
  return `${status} ${result.label} (${result.commandCount} command(s)${duration})${error}`;
}

export function aggregateSmokeReports(results: readonly SmokeTestResult[]): SmokeTestResult[] {
  const reports = new Map<string, SmokeTestResult>();

  for (const result of results) {
    const report = getSmokeReport(reports, result);
    mergeSmokeReportResult(report, result);
  }

  return sortSmokeReports(reports);
}

function getSmokeReport(reports: Map<string, SmokeTestResult>, result: SmokeTestResult): SmokeTestResult {
  const reportId = result.reportId ?? result.id ?? result.label;
  const report = reports.get(reportId);
  if (report) {
    return report;
  }

  const createdReport = createSmokeReportResult(result, reportId, reports.size);
  reports.set(reportId, createdReport);
  return createdReport;
}

function createSmokeReportResult(result: SmokeTestResult, reportId: string, reportOrder: number): SmokeTestResult {
  const reportLabel = result.reportLabel ?? result.label;
  return {
    id: reportId,
    label: reportLabel,
    reportId,
    reportLabel,
    reportOrder: result.reportOrder ?? reportOrder,
    ok: true,
    commandCount: 0,
    durationMs: 0,
    startedAtMs: result.startedAtMs,
    endedAtMs: result.endedAtMs
  };
}

function mergeSmokeReportResult(report: SmokeTestResult, result: SmokeTestResult) {
  report.ok &&= result.ok;
  report.commandCount += result.commandCount;
  report.startedAtMs = Math.min(report.startedAtMs, result.startedAtMs);
  report.endedAtMs = Math.max(report.endedAtMs, result.endedAtMs);
  report.durationMs = report.endedAtMs - report.startedAtMs;
  if (!result.ok && !report.error) {
    report.error = result.error;
  }
}

function sortSmokeReports(reports: Map<string, SmokeTestResult>): SmokeTestResult[] {
  return [...reports.values()].sort((left, right) => (left.reportOrder ?? 0) - (right.reportOrder ?? 0));
}
