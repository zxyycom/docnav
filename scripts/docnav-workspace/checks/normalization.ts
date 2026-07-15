import { expandTasks } from "../../tools/parallel-task-runner/src/index.ts";
import { isStringArray, isUnknownArray } from "../../tools/foundation/src/type-guards.ts";
import type {
  NormalizedTask,
  TaskDefinition
} from "../../tools/parallel-task-runner/src/index.ts";
import type { CheckDefinition, CheckReportRef, CheckTask } from "./model.ts";

export function defineChecks(checkList: readonly CheckDefinition[]): CheckTask[] {
  return withCheckReportMetadata(checkList).map(asCheckTask);
}

function withCheckReportMetadata(checkList: readonly CheckDefinition[]): NormalizedTask[] {
  return expandTasks(checkList.map((check) => annotateCheckReport(check, null)));
}

function annotateCheckReport(check: CheckDefinition, inheritedReport: CheckReportRef | null): TaskDefinition {
  const report = inheritedReport ?? (typeof check.label === "string" ? createCheckReport(check) : null);
  const childChecks = check.tasks;
  if (childChecks !== undefined) {
    const command: unknown = (check as { command?: unknown }).command;
    if (command !== undefined) {
      throw new Error(`check ${check.id} group must not define command`);
    }
    const maybeChildChecks: unknown = childChecks;
    if (!Array.isArray(maybeChildChecks) || childChecks.length === 0) {
      throw new Error(`check ${check.id} tasks must be a non-empty array`);
    }
    return {
      ...check,
      tasks: childChecks.map((child) => annotateCheckReport(child, report))
    };
  }
  const leafReport = report ?? createCheckReport(check);
  return {
    ...check,
    reportId: leafReport.id,
    reportLabel: leafReport.label
  };
}

function createCheckReport(check: CheckDefinition): CheckReportRef {
  return {
    id: check.id,
    label: check.label ?? check.id
  };
}

export function asCheckTask(task: NormalizedTask): CheckTask {
  const allowOutput = isRegExpArray(task.allowOutput) ? task.allowOutput : [];
  const args = isStringArray(task.args) ? task.args : [];
  const command = commandForCheck(task);
  const ignoreOutput = isRegExpArray(task.ignoreOutput) ? task.ignoreOutput : [];
  const warningOutput = isRegExpArray(task.warningOutput) ? task.warningOutput : [];
  return {
    ...task,
    allowOutput,
    args,
    command,
    ignoreOutput,
    warningOutput
  };
}

function commandForCheck(task: NormalizedTask): string {
  if (typeof task.command !== "string" || task.command.trim().length === 0) {
    throw new Error(`check ${task.id} leaf must define a non-empty command`);
  }
  return task.command;
}

function isRegExpArray(value: unknown): value is RegExp[] {
  return isUnknownArray(value) && value.every((item) => item instanceof RegExp);
}
