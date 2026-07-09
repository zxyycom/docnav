import { expandTasks } from "../../../scripts/tools/parallel-task-runner/src/index.ts";
import type { NormalizedTask, TaskDefinition } from "../../../scripts/tools/parallel-task-runner/src/index.ts";
export {
  aggregateSmokeReports,
  formatTestResult,
  withSmokeTaskMetadata
} from "./reports.ts";
export type { SmokeTestResult } from "./reports.ts";

export interface SmokeTask extends TaskDefinition {
  label: string;
  reportId?: string;
  reportLabel?: string;
  reportOrder?: number;
  run?: (task: NormalizedTask) => unknown | Promise<unknown>;
}

export interface SmokeTaskOptions {
  concurrency?: string | number | null;
}

export function prepareSmokeTasks(tasks: readonly SmokeTask[]): NormalizedTask[] {
  return withSmokeReportMetadata(tasks);
}

export function resolveSmokeConcurrency(
  value: string | number | null | undefined
): number | undefined {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isFinite(parsed) || parsed < 1 || String(parsed) !== String(value)) {
    throw new Error(`DOCNAV_SMOKE_CONCURRENCY must be a positive integer: ${value}`);
  }
  return parsed;
}

function withSmokeReportMetadata(tasks: readonly SmokeTask[]): NormalizedTask[] {
  let reportOrder = 0;
  return expandTasks(tasks.map((task) => annotateSmokeReport(task, null, () => reportOrder++)));
}

function annotateSmokeReport(
  task: SmokeTask,
  inheritedReport: { id: string; label: string; order: number } | null,
  nextReportOrder: () => number
): SmokeTask {
  const report = inheritedReport ?? createSmokeReport(task, nextReportOrder);
  if (Array.isArray(task.tasks)) {
    return {
      ...task,
      tasks: task.tasks.map((child) => annotateSmokeReport(child as SmokeTask, report, nextReportOrder))
    };
  }
  return {
    ...task,
    reportId: report.id,
    reportLabel: report.label,
    reportOrder: report.order
  };
}

function createSmokeReport(task: SmokeTask, nextReportOrder: () => number) {
  return {
    id: task.id,
    label: task.label ?? task.id,
    order: nextReportOrder()
  };
}
