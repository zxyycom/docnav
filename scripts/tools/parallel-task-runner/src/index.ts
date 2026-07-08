import { parsePositiveInteger } from "../../foundation/src/index.ts";
import {
  normalizeTaskList,
  type NormalizedTask,
  type TaskDefinition
} from "./tasks/planning.ts";
import { validateTaskGraph } from "./tasks/graph.ts";
import { runTaskScheduler } from "./tasks/scheduler.ts";

export {
  expandTasks,
  normalizeTask,
  type NormalizedTask,
  type StringList,
  type TaskDefinition,
  type TaskEnv
} from "./tasks/planning.ts";
export { validateTaskGraph } from "./tasks/graph.ts";

interface RunParallelTaskOptions<TResult> {
  prepareTasks?: (taskList: readonly TaskDefinition[]) => NormalizedTask[];
  execute?: (task: NormalizedTask) => TResult | Promise<TResult>;
  onStart?: (task: NormalizedTask) => unknown | Promise<unknown>;
  onComplete?: (result: TResult, task: NormalizedTask) => unknown | Promise<unknown>;
  concurrency?: string | number | null;
}

export async function runParallelTasks<TResult = unknown>(
  taskList: readonly TaskDefinition[],
  options: RunParallelTaskOptions<TResult> = {}
): Promise<TResult[]> {
  const pending = prepareTaskQueue(taskList, options.prepareTasks);
  const concurrency = resolveConcurrency(options.concurrency, pending.length);

  return runTaskScheduler({
    pending,
    concurrency,
    execute: options.execute ?? (executeTask as (task: NormalizedTask) => TResult | Promise<TResult>),
    onStart: options.onStart ?? noop,
    onComplete: options.onComplete ?? noop
  });
}

function prepareTaskQueue(
  taskList: readonly TaskDefinition[],
  prepareTasks: ((taskList: readonly TaskDefinition[]) => NormalizedTask[]) | undefined
): NormalizedTask[] {
  const pending = (prepareTasks ?? normalizeTaskList)(taskList);
  validateTaskGraph(pending);
  return pending;
}

function resolveConcurrency(value: string | number | null | undefined, taskCount: number): number {
  if (value === undefined || value === null) {
    return taskCount;
  }

  return parsePositiveInteger(value, "task concurrency");
}

function executeTask(task: NormalizedTask): unknown {
  if (typeof task.run !== "function") {
    throw new Error(`task ${task.id} has no run function`);
  }
  return task.run(task);
}

function noop(): void {}
